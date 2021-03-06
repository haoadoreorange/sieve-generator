mod filter_generator;

use crate::{
    common::GLOBAL_OPTIONS,
    types::{FullFilter, Retirable, SieveDomainConfig, StringOrArray},
};
use filter_generator::FilterGenerator;
use regex::Regex;

fn path_to_prefix_generic_localpart(path: &str) -> String {
    Regex::new(r"/")
        .unwrap()
        .replace_all(&Regex::new(r"\s+").unwrap().replace_all(path, "-"), ".")
        .into_owned()
        .to_lowercase()
        + "."
}

pub struct DomainGenerator<'a> {
    custom_filter_generator: FilterGenerator<'a>,
    generic_filter_generator: FilterGenerator<'a>,
    secrets: Vec<String>,
}

impl DomainGenerator<'_> {
    pub fn new(domain: &str, secrets: Vec<String>, domain_as_first_folder: bool) -> Self {
        let domain_folder = if GLOBAL_OPTIONS.lock().unwrap().force_domain_as_first_folder
            || domain_as_first_folder
        {
            format!("@{}/", domain)
        } else {
            "".to_string()
        };
        DomainGenerator {
            custom_filter_generator: FilterGenerator::new(
                "Custom Filters",
                domain_folder.clone(),
                false,
            ),
            generic_filter_generator: FilterGenerator::new("Generic Filters", domain_folder, true),
            secrets,
        }
    }

    fn _generate(
        &mut self,
        sub_config: SieveDomainConfig,
        path: &str,
        fakeroot_path: &str,
        mut parent_silent: bool,
    ) {
        let mut skip_generic = false;
        let mut silent = None; // for generic filter
        let mut labels = None;
        if path == "Unknown" {
            parent_silent = true; // Everything in Unknown is silent
            skip_generic = true; // No need to have generic filter for Unknown
        }

        // Custom filter
        match sub_config {
            SieveDomainConfig::SimpleFilter(localparts) => {
                self.custom_filter_generator.generate(
                    path,
                    FullFilter {
                        localparts,
                        labels: None,
                        silent: Some(parent_silent),
                        generic: None,
                    },
                );
            }
            SieveDomainConfig::FullFilter(mut full_filter) => {
                skip_generic = !full_filter.generic.unwrap_or(true);
                full_filter.silent = Some(full_filter.silent.unwrap_or(parent_silent));
                silent = full_filter.silent;
                labels = full_filter.labels.clone();
                self.custom_filter_generator.generate(path, full_filter);
            }
            SieveDomainConfig::Object(mut o) => {
                if o.is_empty() {
                    // TODO: Add test
                    panic!("This is an empty sieve config, are you high ?");
                }
                let mut fakeroot = false;
                if let Some(SieveDomainConfig::Boolean(b)) = o.remove("fakeroot") {
                    fakeroot = b;
                }
                if let Some(SieveDomainConfig::Boolean(b)) = o.remove("silent") {
                    parent_silent = b; // TODO: add test parent silent
                }
                if o.is_empty() {
                    // TODO: Add test
                    panic!("Hm...fakeroot & silent for an empty sieve config, are you high ?");
                }
                let sub_config_length = o.len();
                for (sub, next_sub_config) in o.drain() {
                    if sub.is_empty() {
                        panic!("Oups...the empty string cannot be used for folder name");
                    }
                    let tmp: String;
                    let new_path = if path.is_empty() {
                        if sub == "self" {
                            panic!("Sorry baby ): 'self' field is not supported at domain level");
                        }
                        &sub
                    } else if sub == "self" {
                        if sub_config_length == 1 {
                            // TODO: Add test
                            panic!("Hm...Why use 'self' if there is no sub-folder, are you high ?");
                        }
                        // if self field exist, generic filter generator for current path will be run again
                        // in next recursive with more detailed info BEFORE the current recursive,
                        // hence making the current obsolete. Not skipping it will result in
                        // obsolete filter overwrite more detailed filter
                        // TODO: add test for this case
                        skip_generic = true;
                        path
                    } else {
                        tmp = format!("{}/{}", path, sub);
                        &tmp
                    };
                    // fakeroot only take the latest fakeroot
                    // TODO: add parents's fakeroot
                    self._generate(
                        next_sub_config,
                        new_path,
                        if fakeroot && sub != "self" { &sub } else { "" },
                        parent_silent,
                    );
                }
            }
            _ => {
                panic!("Unable to handle {}", path);
            }
        }

        // Generic filter
        if !path.is_empty() && !skip_generic {
            let prefix_generic_lps = if !fakeroot_path.is_empty() {
                vec![path_to_prefix_generic_localpart(fakeroot_path)]
            } else {
                vec![path_to_prefix_generic_localpart(path)]
            };
            self.generic_filter_generator.generate(
                path,
                FullFilter {
                    localparts: StringOrArray::Array(
                        self.secrets
                            .iter()
                            .map(|secret| {
                                prefix_generic_lps
                                    .iter()
                                    .map(|lp| vec![lp.clone() + secret, lp.clone() + secret + ".*"])
                                    .collect::<Vec<_>>()
                                    .concat()
                            })
                            .collect::<Vec<_>>()
                            .concat(),
                    ),
                    labels,
                    silent: Some(silent.unwrap_or(parent_silent)),
                    generic: None,
                },
            );
        }
    }

    pub fn generate(&mut self, sieve_domain_config: SieveDomainConfig) -> &mut Self {
        self._generate(sieve_domain_config, "", "", false);
        self
    }
}

impl Retirable for DomainGenerator<'_> {
    fn retire(self) -> String {
        self.custom_filter_generator.retire()
            + &self.generic_filter_generator.retire_with_unknown_filter()
    }
}

#[cfg(test)]
mod tests {
    use super::Retirable;

    #[test]
    fn path_to_prefix_generic_localpart() {
        assert_eq!(
            super::path_to_prefix_generic_localpart("Home bills/Electricity"),
            "home-bills.electricity.",
        );
    }

    #[test]
    fn domain_generator() {
        let mut g = super::DomainGenerator::new("domain", vec!["slyth".to_string()], false);
        g.generate(
            serde_json::from_str::<super::SieveDomainConfig>(
                r#"
                        {
                            "Finance": {
                                "fakeroot": true,
                                "Stock markets": ["broker1", "broker2"],
                                "Bank" : {
                                    "localparts": "bank-account",
                                    "labels": {
                                        "statement": ["statement"]
                                    }
                                },
                                "Bank2" : {
                                    "localparts": "",
                                    "labels": {
                                        "statement": ["statement"]
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
            g.retire(),
            r#"
# Custom Filters
if envelope :localpart :matches "to" ["bank-account"] {
    fileinto "Finance";
    fileinto "Finance/Bank";
    if header :contains "subject" ["statement"] {
        fileinto "statement";
    }
} elsif envelope :localpart :matches "to" ["broker1","broker2"] {
    fileinto "Finance";
    fileinto "Finance/Stock markets";
} elsif envelope :localpart :matches "to" ["wallstreet"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
}
# Generic Filters
elsif envelope :localpart :matches "to" ["finance.slyth","finance.slyth.*"] {
    fileinto "Finance";
} elsif envelope :localpart :matches "to" ["bank.slyth","bank.slyth.*"] {
    fileinto "Finance";
    fileinto "Finance/Bank";
    if header :contains "subject" ["statement"] {
        fileinto "statement";
    }
} elsif envelope :localpart :matches "to" ["bank2.slyth","bank2.slyth.*"] {
    fileinto "Finance";
    fileinto "Finance/Bank2";
    if header :contains "subject" ["statement"] {
        fileinto "statement";
    }
} elsif envelope :localpart :matches "to" ["stock-markets.slyth","stock-markets.slyth.*"] {
    fileinto "Finance";
    fileinto "Finance/Stock markets";
} elsif envelope :localpart :matches "to" ["newsletter.slyth","newsletter.slyth.*"] {
    fileinto "Newsletter";
} elsif envelope :localpart :matches "to" ["newsletter.business.slyth","newsletter.business.slyth.*"] {
    fileinto "Newsletter";
    fileinto "Newsletter/Business";
} else {
    fileinto "Unknown";
    addflag "\\Seen";
}"#
        );
    }

    #[test]
    #[should_panic(expected = "'self' field is not supported at domain level")]
    fn domain_generator_panic_self_domain() {
        super::DomainGenerator::new("domain", vec!["slyth".to_string()], false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"self": "self"}"#).unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "Unable to handle")]
    fn domain_generator_panic_unable_to_handle() {
        super::DomainGenerator::new("domain", vec!["slyth".to_string()], false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"folder1": { "key": true } }"#)
                .unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "empty string cannot be used")]
    fn domain_generator_panic_folder_cannot_be_empty_string() {
        super::DomainGenerator::new("domain", vec!["slyth".to_string()], false).generate(
            serde_json::from_str::<super::SieveDomainConfig>(r#"{"folder1": { "": true } }"#)
                .unwrap(),
        );
    }
}
