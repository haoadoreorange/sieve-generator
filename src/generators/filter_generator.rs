use crate::{
    common::{code_block, panic_on_empty_array_string},
    types::{FullFilter, PanicOnEmpty, Retirable, StringOrArray},
};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub struct FilterGenerator<'a> {
    name: &'a str,
    domain_folder: String,
    filters: BTreeMap<String, FullFilter<Vec<String>>>,
    begin_with_else: bool,
}

impl<'a> FilterGenerator<'a> {
    pub fn new(name: &'a str, domain_folder: String, begin_with_else: bool) -> FilterGenerator<'a> {
        FilterGenerator {
            name,
            domain_folder,
            filters: BTreeMap::new(),
            begin_with_else,
        }
    }

    pub fn generate(&mut self, path: &str, full_filter: FullFilter<StringOrArray>) -> &mut Self {
        if path.is_empty() {
            panic!(
                "BUG: Folder path is empty, something is wrong with {}",
                self.name
            );
        }

        let localparts = match full_filter.localparts {
            StringOrArray::String(localpart) => {
                if localpart.is_empty() {
                    // Filters localpart can take an empty string to use only generic
                    return self;
                } else {
                    vec![localpart]
                }
            }
            StringOrArray::Array(localparts) => {
                panic_on_empty_array_string(localparts, "localparts")
            }
        };

        let labels = if let Some(full_filter_labels) = full_filter.labels {
            let mut labels = BTreeMap::new();
            for (label, keywords) in full_filter_labels.into_iter() {
                if label.is_empty() {
                    panic!("Label cannot be empty string");
                }
                labels.insert(
                    label,
                    match keywords.panic_on_empty("label keywords") {
                        StringOrArray::String(keyword) => vec![keyword],
                        StringOrArray::Array(keywords) => keywords,
                    },
                );
            }
            Some(labels)
        } else {
            None
        };

        self.filters.insert(
            path.to_string(),
            FullFilter {
                localparts,
                labels,
                silent: full_filter.silent,
                generic: None,
            },
        );
        self
    }

    fn transform_to_string(self) -> String {
        let mut result = "".to_string();
        for (i, (path, full_filter)) in self.filters.into_iter().enumerate() {
            result = result
                + &if i == 0 {
                    format!(
                        "\n# {}\n{}if",
                        self.name,
                        if self.begin_with_else { "els" } else { "" }
                    )
                } else {
                    " elsif".to_string()
                }
                + " envelope :localpart :matches \"to\" "
                + &serde_json::to_string(&full_filter.localparts).unwrap()
                + " {"
                + &code_block({
                    let mut cumulated_path = "".to_string();
                    let mut file_into = "".to_string();
                    for folder in path.split('/').collect::<Vec<_>>() {
                        if cumulated_path.is_empty() {
                            cumulated_path = folder.to_string();
                        } else {
                            cumulated_path = format!("{}/{}", cumulated_path, folder);
                        }
                        if cumulated_path != "Unknown" {
                            file_into = file_into
                                + &format!(
                                    "\nfileinto \"{}{}\";",
                                    self.domain_folder, cumulated_path
                                )
                        }
                    }
                    file_into
                })
                + &code_block({
                    let mut labels = "".to_string();
                    let mut all_keywords = HashSet::new();
                    let mut multiple_labels = false;
                    if let Some(full_filter_labels) = full_filter.labels {
                        if full_filter_labels.len() > 1 {
                            multiple_labels = true;
                        }
                        for (label, keywords) in full_filter_labels.into_iter() {
                            labels = labels
                                + "\nif header :contains \"subject\" "
                                + &serde_json::to_string(&keywords).unwrap()
                                + " {"
                                + &code_block(format!("\nfileinto \"{}\";", label))
                                + "\n}";
                            if full_filter.silent.unwrap_or(false) && multiple_labels {
                                keywords.iter().for_each(|keyword| {
                                    all_keywords.insert(keyword.clone());
                                })
                            }
                        }
                    }
                    let silent = "\naddflag \"\\\\Seen\";\nfileinto \"unread\";".to_string();
                    if full_filter.silent.unwrap_or(false) {
                        // TODO: add test silent
                        if !labels.is_empty() {
                            (if multiple_labels {
                                // TODO: Add test case
                                "\nif header :contains \"subject\" ".to_string()
                                    + &serde_json::to_string(&all_keywords).unwrap()
                                    + " {"
                                    + &code_block(labels)
                                    + "\n}"
                            } else {
                                labels
                            }) + " else {"
                                + &code_block(silent)
                                + "\n}"
                        } else {
                            silent
                        }
                    } else {
                        labels
                    }
                })
                + "\n}";
        }
        result
    }

    pub fn retire_with_unknown_filter(self) -> String {
        self.retire()
            + " else {"
            + &code_block("\nfileinto \"Unknown\";\naddflag \"\\\\Seen\";")
            + "\n}"
    }
}

impl Retirable for FilterGenerator<'_> {
    fn retire(self) -> String {
        self.transform_to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Retirable;

    // TODO: add silent test
    #[test]
    fn filter_generator() {
        let mut fg =
            super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false);
        fg.generate(
            "Home bills/electricity",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec![
                    "home-bills.electricity".to_string(),
                    "custom".to_string(),
                ]),
                labels: None,
                silent: None,
                generic: None,
            },
        )
        .generate(
            "Home bills/grocery",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec![
                    "home-bills.grocery".to_string(),
                    "custom".to_string(),
                ]),
                labels: None,
                silent: None,
                generic: None,
            },
        );
        assert_eq!(
            fg.retire_with_unknown_filter(),
            r#"
# Test filter generator
if envelope :localpart :matches "to" ["home-bills.electricity","custom"] {
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/electricity";
} elsif envelope :localpart :matches "to" ["home-bills.grocery","custom"] {
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/grocery";
} else {
    fileinto "Unknown";
    addflag "\\Seen";
}"#
        );
    }

    #[test]
    #[should_panic(expected = "Folder path is empty")]
    fn filter_generator_panic_empty_path() {
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false)
            .generate(
                "",
                super::FullFilter {
                    localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                    labels: None,
                    silent: None,
                    generic: None,
                },
            );
    }

    #[test]
    fn filter_generator_empty_lp() {
        let mut fg =
            super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false);
        fg.generate(
            "path",
            super::FullFilter {
                localparts: super::StringOrArray::String("".to_string()),
                labels: None,
                silent: None,
                generic: None,
            },
        );
        assert_eq!(fg.retire(), "");
    }

    #[test]
    #[should_panic(expected = "Label cannot be empty")]
    fn filter_generator_panic_label_empty() {
        let mut labels = super::BTreeMap::new();
        labels.insert("".to_string(), super::StringOrArray::String("".to_string()));
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false)
            .generate(
                "path",
                super::FullFilter {
                    localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                    labels: Some(labels),
                    silent: None,
                    generic: None,
                },
            );
    }
}
