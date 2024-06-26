mod filter_generator;
use std::fmt;

use crate::common::{is_unknown, FilterOptions, FullFilter, SieveDomainConfig, StringOrVec};
use filter_generator::FilterGenerator;
use regex::Regex;

pub struct DomainGenerator<'a> {
    custom_filter_generator: FilterGenerator<'a>,
    generic_filter_generator: FilterGenerator<'a>,
}

impl DomainGenerator<'_> {
    //
    pub fn new(domain: &str, domain_as_first_folder: bool) -> Self {
        let domain_folder = if domain_as_first_folder {
            format!("@{domain}/")
        } else {
            String::from("")
        };
        DomainGenerator {
            custom_filter_generator: FilterGenerator::new("Custom", domain_folder.clone(), false),
            generic_filter_generator: FilterGenerator::new("Generic", domain_folder, true),
        }
    }

    pub fn generate(&mut self, sieve_domain_config: SieveDomainConfig) -> &mut Self {
        self._generate("", sieve_domain_config)
    }

    fn _generate(&mut self, path: &str, sub_config: SieveDomainConfig) -> &mut Self {
        /* Also need for generic filter. */
        let mut labels = None;
        let mut options = if !is_unknown(path) {
            FilterOptions::<bool> {
                generic: true,       // Default
                fullpath: false,     // Default
                mark_as_read: false, // Default
            }
        } else {
            FilterOptions::<bool> {
                generic: false,     // No generic filter for Unknown.
                fullpath: false,    // Ignored
                mark_as_read: true, // Everything under Unknown is marked as read.
            }
        };
        /********************************/

        /* Custom filter */
        match sub_config {
            SieveDomainConfig::SimpleFilter(localparts) => {
                self.custom_filter_generator.generate(
                    path,
                    FullFilter {
                        localparts,
                        labels: None,
                        options,
                    },
                );
            }
            SieveDomainConfig::FullFilter(full_filter) => {
                labels = full_filter.labels.clone();
                if let Some(full_filter_options) = full_filter.options {
                    options = full_filter_options.unwrap_or_default(options);
                    if full_filter_options.fullpath.is_some() {
                        if !options.generic {
                            panic!(
                                "ERROR: Not generating generic filters for {}, set fullpath option is useless.",
                                path
                            );
                        }
                        if !path.contains('/') {
                            panic!(
                                "ERROR: {} is the whole path, set fullpath option is useless.",
                                path
                            );
                        }
                    }
                }
                self.custom_filter_generator.generate(
                    path,
                    FullFilter::<StringOrVec, FilterOptions<bool>> {
                        localparts: full_filter.localparts,
                        labels: full_filter.labels,
                        options,
                    },
                );
            }
            SieveDomainConfig::SubDomainConfig(mut sub_domain_configs) => {
                if sub_domain_configs.is_empty() {
                    panic!("ERROR: Found an empty sub-domain config, are you high ?");
                }

                let sub_domain_configs_len = sub_domain_configs.len();
                for (sub, next_sub_config) in sub_domain_configs.drain() {
                    if sub.is_empty() {
                        panic!("ERROR: Oups...empty string cannot be used for folder name.");
                    }
                    let tmp: String;
                    let new_path = if path.is_empty() {
                        if sub == "self" {
                            panic!("ERROR: Sorry baby ): 'self' field is not supported at domain level.");
                        }
                        &sub
                    } else if sub == "self" {
                        if sub_domain_configs_len == 1 {
                            panic!("ERROR: Hm...Why use 'self' if there is no sub-folder, are you high ?");
                        }
                        /*
                         * If self field exist, generic filter generator for current path will be run again
                         * in next recursive with more detailed info BEFORE the current recursive,
                         * hence making the current obsolete. Not skipping it will result in
                         * obsolete filter overwrite more detailed filter.
                         */
                        options.generic = false;
                        path
                    } else {
                        tmp = format!("{}/{}", path, sub);
                        &tmp
                    };
                    self._generate(new_path, next_sub_config);
                }
            }
        }

        /* Generic filter, path is empty first recursive. */
        if options.generic && !path.is_empty() {
            let prefix_generic_lps = path_to_prefix_generic_localpart(if !options.fullpath {
                filename_of(path)
            } else {
                path
            });
            self.generic_filter_generator.generate(
                path,
                FullFilter::<StringOrVec, FilterOptions<bool>> {
                    localparts: StringOrVec::Vec(vec![
                        prefix_generic_lps.clone(),
                        prefix_generic_lps + ".*",
                    ]),
                    labels,
                    options,
                },
            );
        }
        self
    }
}

impl fmt::Display for DomainGenerator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.custom_filter_generator.to_string()
                + &self.generic_filter_generator.to_string_with_unknown()
        )
    }
}

/*
 * "A B"/C -> a-b.c
 */
fn path_to_prefix_generic_localpart(path: &str) -> String {
    Regex::new(r"/")
        .unwrap()
        .replace_all(&Regex::new(r"\s+").unwrap().replace_all(path, "-"), ".")
        .into_owned()
        .to_lowercase()
}

/*
 * A/B/C -> C
 */
fn filename_of(path: &str) -> &str {
    if path.contains('/') {
        Regex::new(r".*/(.+)$")
            .unwrap()
            .captures(path)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
    } else {
        path
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
        assert_eq!(super::filename_of("Home bills/Electricity"), "Electricity",);
    }

    #[test]
    fn domain_generator() {
        let mut g = super::DomainGenerator::new("domain", false);
        g.generate(
            serde_json::from_str::<super::SieveDomainConfig>(
                r#"
                    {
                        "Newsletter": {
                            "Software": ["google", "facebook"],
                            "Business": {
                                "localparts": "wallstreet",
                                "options": {
                                    "fullpath": true
                                }
                            }
                        },
                        "Utilities": {
                            "self": {
                                "localparts": ["electricity"],
                                "options": {
                                    "generic": false
                                }
                            },
                            "Grocery": {
                                "localparts": "market",
                                "labels": {
                                    "label": "keyword"
                                },
                                "options": {
                                    "mark-as-read": true
                                }
                            },
                            "Bill": {
                                "localparts": "",
                                "labels": {
                                    "label2": ["keyword2"],
                                    "label3": ["keyword3"]
                                },
                                "options": {
                                    "mark-as-read": true
                                }
                            }
                        }
                    }"#,
            )
            .unwrap(),
        );
        assert_eq!(
            g.to_string(),
            r#"
# Custom filters
if envelope :localpart :matches "to" ["market"] {
    fileinto "Utilities";
    fileinto "Utilities/Grocery";
    if header :contains ["from","subject"] ["keyword"] {
        fileinto "label";
    } else {
        addflag "\\Seen";
        fileinto "unread";
    }
} elsif envelope :localpart :matches "to" ["electricity"] {
    fileinto "Utilities";
} elsif envelope :localpart :matches "to" ["google","facebook"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Software";
} elsif envelope :localpart :matches "to" ["wallstreet"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
}
# Generic filters
elsif envelope :localpart :matches "to" ["grocery","grocery.*"] {
    fileinto "Utilities";
    fileinto "Utilities/Grocery";
    if header :contains ["from","subject"] ["keyword"] {
        fileinto "label";
    } else {
        addflag "\\Seen";
        fileinto "unread";
    }
} elsif envelope :localpart :matches "to" ["bill","bill.*"] {
    fileinto "Utilities";
    fileinto "Utilities/Bill";
    if header :contains ["from","subject"] ["keyword2","keyword3"] {
        if header :contains ["from","subject"] ["keyword2"] {
            fileinto "label2";
        }
        if header :contains ["from","subject"] ["keyword3"] {
            fileinto "label3";
        }
    } else {
        addflag "\\Seen";
        fileinto "unread";
    }
} elsif envelope :localpart :matches "to" ["software","software.*"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Software";
} elsif envelope :localpart :matches "to" ["newsletter.business","newsletter.business.*"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
} elsif envelope :localpart :matches "to" ["newsletter","newsletter.*"] {
    fileinto "Newsletter";
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
            g.to_string(),
            r#"
# Custom filters
if envelope :localpart :matches "to" ["wallstreet"] {
    fileinto "@domain/Newsletter";
    fileinto "@domain/Newsletter/Business";
}
# Generic filters
elsif envelope :localpart :matches "to" ["business","business.*"] {
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
