use crate::{
    common::{code_block, panic_on_empty},
    types::{FullFilter, PanicOnEmpty, Retirable, StringOrArray},
};
use std::{collections::BTreeMap, fmt};

#[derive(Debug)]
pub struct FilterGenerator<'a> {
    name: &'a str,
    domain_folder: String,
    filters: BTreeMap<String, FullFilter<Vec<String>>>,
}

impl<'a> FilterGenerator<'a> {
    pub fn new(name: &'a str, domain_folder: String) -> FilterGenerator<'a> {
        FilterGenerator {
            name,
            domain_folder,
            filters: BTreeMap::new(),
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
            StringOrArray::Array(localparts) => panic_on_empty(localparts, "localparts"),
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

        self.filters
            .insert(path.to_string(), FullFilter { localparts, labels });
        self
    }

    pub fn retire_with_unknown_filter(self) -> String {
        self.retire() + " else {" + &code_block("\nfileinto \"Unknown\";") + "\n}"
    }
}

impl Retirable for FilterGenerator<'_> {
    fn retire(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for FilterGenerator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "".to_string();
        for (i, (path, full_filter)) in self.filters.iter().enumerate() {
            result = result
                + &if i == 0 {
                    format!("\n# {}\nif", self.name)
                } else {
                    " elsif".to_string()
                }
                + " envelope :localpart :matches \"to\" "
                + &serde_json::to_string(&full_filter.localparts).unwrap()
                + " {"
                + &code_block({
                    let mut cumulated_path = "".to_string();
                    let mut result = "".to_string();
                    for folder in path.split('/').collect::<Vec<_>>() {
                        if cumulated_path.is_empty() {
                            cumulated_path = folder.to_string();
                        } else {
                            cumulated_path = format!("{}/{}", cumulated_path, folder);
                        }
                        result = result
                            + &format!("\nfileinto \"{}{}\";", self.domain_folder, cumulated_path)
                    }
                    result
                })
                + &code_block({
                    let mut result = "".to_string();
                    if let Some(full_filter_labels) = &full_filter.labels {
                        for (label, keywords) in full_filter_labels.iter() {
                            result = result
                                + "\nif header :contains \"subject\" "
                                + &serde_json::to_string(keywords).unwrap()
                                + " {"
                                + &code_block(format!("\nfileinto \"{}\";", label))
                                + "\n}";
                        }
                    }
                    result
                })
                + "\n}";
        }
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::Retirable;

    #[test]
    fn filter_generator() {
        let mut fg = super::FilterGenerator::new("Test filter generator", "@domain/".to_string());
        fg.generate(
            "Home bills/electricity",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec![
                    "home-bills.electricity".to_string(),
                    "custom".to_string(),
                ]),
                labels: None,
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
}"#
        );
    }

    #[test]
    #[should_panic(expected = "Folder path is empty")]
    fn filter_generator_panic_empty_path() {
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string()).generate(
            "",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                labels: None,
            },
        );
    }

    #[test]
    fn filter_generator_empty_lp() {
        let mut fg = super::FilterGenerator::new("Test filter generator", "@domain/".to_string());
        fg.generate(
            "path",
            super::FullFilter {
                localparts: super::StringOrArray::String("".to_string()),
                labels: None,
            },
        );
        assert_eq!(fg.retire(), "");
    }

    #[test]
    #[should_panic(expected = "Label cannot be empty")]
    fn filter_generator_panic_label_empty() {
        let mut labels = super::BTreeMap::new();
        labels.insert("".to_string(), super::StringOrArray::String("".to_string()));
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string()).generate(
            "path",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                labels: Some(labels),
            },
        );
    }
}
