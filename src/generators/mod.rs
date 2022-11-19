mod filter_generator;

use crate::types::{FilterOptions, FullFilter, SieveDomainConfig, StringOrArray};
use filter_generator::FilterGenerator;
use regex::Regex;

// "A B"/C -> a-b.c
fn path_to_prefix_generic_localpart(path: &str) -> String {
    Regex::new(r"/")
        .unwrap()
        .replace_all(&Regex::new(r"\s+").unwrap().replace_all(path, "-"), ".")
        .into_owned()
        .to_lowercase()
}

// A/B/C -> C
fn last_folder_of_path(path: &str) -> &str {
    Regex::new(r".*/(.+)$")
        .unwrap()
        .captures(path)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
}

pub struct DomainGenerator<'a> {
    custom_filter_generator: FilterGenerator<'a>,
    generic_filter_generator: FilterGenerator<'a>,
}

impl DomainGenerator<'_> {
    pub fn new(domain: &str, domain_as_first_folder: bool) -> Self {
        let domain_folder = if domain_as_first_folder {
            format!("@{domain}/")
        } else {
            "".to_string()
        };
        DomainGenerator {
            custom_filter_generator: FilterGenerator::new("Custom", domain_folder.clone(), false),
            generic_filter_generator: FilterGenerator::new("Generic", domain_folder, true),
        }
    }

    fn _generate(
        &mut self,
        path: &str,
        sub_config: SieveDomainConfig,
        // Everything below Unknown is silent, we need this to pass options down to children
        mut inherited_options: Option<FilterOptions>,
    ) -> &mut Self {
        let mut labels = None; // Need to store this for generic filter also
        let mut options = inherited_options.clone().unwrap_or(FilterOptions {
            generic: Some(true), // Default
            orphan: Some(false), // Default
            silent: None,        // No need to care at this level
        }); // Need the options for generic filter also

        if path == "Unknown" {
            inherited_options = Some(options.mask_with(&FilterOptions {
                generic: Some(false), // No need to have generic filter for Unknown
                orphan: None,
                silent: Some(true), // Everything in Unknown is silent
            }));
            options = inherited_options.clone().unwrap();
        }

        // Custom filter
        match sub_config {
            SieveDomainConfig::SimpleFilter(localparts) => {
                self.custom_filter_generator.generate(
                    path,
                    FullFilter {
                        localparts,
                        labels: labels.clone(),
                        options: inherited_options,
                    },
                );
            }
            SieveDomainConfig::FullFilter(mut full_filter) => {
                labels = full_filter.labels.clone();
                if let Some(full_filter_options) = full_filter.options {
                    options = options.mask_with(&full_filter_options);
                }
                full_filter.options = Some(options.clone());
                self.custom_filter_generator.generate(path, full_filter);
            }
            SieveDomainConfig::SubDomainConfig(mut o) => {
                if o.is_empty() {
                    panic!("This is an empty sieve config, are you high ?");
                }

                let sub_config_length = o.len();
                for (sub, next_sub_config) in o.drain() {
                    if sub.is_empty() {
                        panic!("Oups...empty string cannot be used for folder name");
                    }
                    let tmp: String;
                    let new_path = if path.is_empty() {
                        if sub == "self" {
                            panic!("Sorry baby ): 'self' field is not supported at domain level");
                        }
                        &sub
                    } else if sub == "self" {
                        if sub_config_length == 1 {
                            panic!("Hm...Why use 'self' if there is no sub-folder, are you high ?");
                        }
                        // if self field exist, generic filter generator for current path will be run again
                        // in next recursive with more detailed info BEFORE the current recursive,
                        // hence making the current obsolete. Not skipping it will result in
                        // obsolete filter overwrite more detailed filter
                        options.generic = Some(false);
                        path
                    } else {
                        tmp = format!("{}/{}", path, sub);
                        &tmp
                    };
                    self._generate(new_path, next_sub_config, inherited_options.clone());
                }
            }
        }

        // Generic filter
        if !path.is_empty() && options.generic.unwrap() {
            let prefix_generic_lps = vec![path_to_prefix_generic_localpart(
                if options.orphan.unwrap() {
                    last_folder_of_path(path)
                } else {
                    path
                },
            )];
            self.generic_filter_generator.generate(
                path,
                FullFilter {
                    localparts: StringOrArray::Array(
                        prefix_generic_lps
                            .iter()
                            .map(|lp| vec![lp.clone(), lp.clone() + ".*"])
                            .collect::<Vec<_>>()
                            .concat(),
                    ),
                    labels,
                    options: Some(options),
                },
            );
        }
        self
    }

    pub fn generate(&mut self, sieve_domain_config: SieveDomainConfig) -> &mut Self {
        self._generate("", sieve_domain_config, None)
    }
}

impl From<DomainGenerator<'_>> for String {
    fn from(item: DomainGenerator<'_>) -> Self {
        Into::<String>::into(item.custom_filter_generator)
            + &item.generic_filter_generator.into_string_with_unknown()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn path_to_prefix_generic_localpart() {
        assert_eq!(
            super::path_to_prefix_generic_localpart("Home bills/Electricity"),
            "home-bills.electricity",
        );
    }

    #[test]
    fn last_folder_of_path() {
        assert_eq!(
            super::last_folder_of_path("Home bills/Electricity"),
            "Electricity",
        );
    }

    #[test]
    fn domain_generator() {
        let mut g = super::DomainGenerator::new("domain", false);
        g.generate(
            serde_json::from_str::<super::SieveDomainConfig>(
                r#"
                        {
                            "Finance": {
                                "self": {
                                    "localparts": "",
                                    "labels": {
                                        "statement": ["statement"]
                                    }
                                },
                                "Stock markets": {
                                    "localparts": ["broker1", "broker2"],
                                    "options": {
                                        "orphan": true
                                    }
                                },
                                "Bank" : {
                                    "localparts": "bank-account",
                                    "labels": {
                                        "statement": ["statement"]
                                    },
                                    "options": {
                                        "generic": false
                                    }
                                },
                                "Bank2" : {
                                    "localparts": "",
                                    "labels": {
                                        "statement": ["statement"]
                                    },
                                    "options": {
                                        "orphan": true
                                    }
                                }
                            },
                            "Newsletter": {
                                "Business": "wallstreet"
                            } 
                        }"#,
            )
            .unwrap(),
        );
        assert_eq!(
            Into::<String>::into(g),
            r#"
# Custom filters
if envelope :localpart :matches "to" ["wallstreet"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
} elsif envelope :localpart :matches "to" ["broker1","broker2"] {
    fileinto "Finance";
    fileinto "Finance/Stock markets";
} elsif envelope :localpart :matches "to" ["bank-account"] {
    if header :contains ["from","subject"] ["statement"] {
        fileinto "statement";
    }
    fileinto "Finance";
    fileinto "Finance/Bank";
}
# Generic filters
elsif envelope :localpart :matches "to" ["newsletter.business","newsletter.business.*"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
} elsif envelope :localpart :matches "to" ["newsletter","newsletter.*"] {
    fileinto "Newsletter";
} elsif envelope :localpart :matches "to" ["stock-markets","stock-markets.*"] {
    fileinto "Finance";
    fileinto "Finance/Stock markets";
} elsif envelope :localpart :matches "to" ["bank2","bank2.*"] {
    if header :contains ["from","subject"] ["statement"] {
        fileinto "statement";
    }
    fileinto "Finance";
    fileinto "Finance/Bank2";
} elsif envelope :localpart :matches "to" ["finance","finance.*"] {
    if header :contains ["from","subject"] ["statement"] {
        fileinto "statement";
    }
    fileinto "Finance";
} else {
    addflag "\\Seen";
    fileinto "Unknown";
}"#
        );
    }

    #[test]
    fn domain_generator_domain_as_first_folder() {
        let mut g = super::DomainGenerator::new("domain", true);
        g.generate(
            serde_json::from_str::<super::SieveDomainConfig>(
                r#"
                        {
                            "Newsletter": {
                                "Business": "wallstreet"
                            } 
                        }"#,
            )
            .unwrap(),
        );
        assert_eq!(
            Into::<String>::into(g),
            r#"
# Custom filters
if envelope :localpart :matches "to" ["wallstreet"] {
    fileinto "@domain/Newsletter";
    fileinto "@domain/Newsletter/Business";
}
# Generic filters
elsif envelope :localpart :matches "to" ["newsletter.business","newsletter.business.*"] {
    fileinto "@domain/Newsletter";
    fileinto "@domain/Newsletter/Business";
} elsif envelope :localpart :matches "to" ["newsletter","newsletter.*"] {
    fileinto "@domain/Newsletter";
} else {
    addflag "\\Seen";
    fileinto "Unknown";
}"#
        );
    }

    #[test]
    #[should_panic(expected = "are you high")]
    fn domain_generator_panic_empty_config() {
        super::DomainGenerator::new("domain", false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"folder": {} }"#).unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "'self' field is not supported at domain level")]
    fn domain_generator_panic_self_domain() {
        super::DomainGenerator::new("domain", false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"self": "self"}"#).unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "are you high")]
    fn domain_generator_panic_self_with_no_sub() {
        super::DomainGenerator::new("domain", false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"folder": { "self": "" } }"#)
                .unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "empty string cannot be used")]
    fn domain_generator_panic_folder_cannot_be_empty_string() {
        super::DomainGenerator::new("domain", false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"folder1": { "": "" } }"#)
                .unwrap(),
        );
    }
}
