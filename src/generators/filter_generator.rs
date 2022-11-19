use crate::{
    common::code_block,
    types::{FilterOptions, FullFilter, PanicOnEmpty, StringOrArray},
};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub struct FilterGenerator<'a> {
    name: &'a str, // Name of the generator
    domain_folder: String,
    filters: BTreeMap<String, FullFilter<Vec<String>>>,
    begin_with_else: bool, // It can begin with else if generated after another
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
                "BUG: calling {}.generate() with empty folder path, something is wrong",
                self.name
            );
        }
        if let StringOrArray::String(localpart) = &full_filter.localparts {
            if localpart.is_empty() {
                // Filters localpart can take an empty string to use only generic
                return self;
            }
        }

        // Now we "clean" the data before put it in the filters list, makes it easier & safe to into() later
        // Convert FullFilter<StringOrArray> to FullFilter<Vec> for easier into String
        let localparts: Vec<String> = full_filter.localparts.panic_on_empty("localparts").into();
        let labels = if let Some(full_filter_labels) = full_filter.labels {
            let mut labels = BTreeMap::new();
            for (label, keywords) in full_filter_labels.into_iter() {
                if label.is_empty() {
                    panic!("Label cannot be empty string");
                }
                labels.insert(label, keywords.panic_on_empty("label keywords").into());
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
                // First set default options, and then mask it with the filter options if there's one
                options: Some({
                    let default = FilterOptions {
                        generic: None,       // No need to care at this level
                        orphan: None,        // No need to care at this level
                        silent: Some(false), // Default
                    };
                    if let Some(options) = full_filter.options {
                        default.mask_with(&options)
                    } else {
                        default
                    }
                }),
            },
        );

        self
    }

    pub fn into_string_with_unknown(self) -> String {
        Into::<String>::into(self)
            + " else {"
            + &code_block("\naddflag \"\\\\Seen\";\nfileinto \"Unknown\";")
            + "\n}"
    }
}

impl From<FilterGenerator<'_>> for String {
    fn from(item: FilterGenerator<'_>) -> Self {
        let mut result = "".to_string();
        // rev() is for generic filter, A/B must be filtered before A, otherwise a.b will all go to A, not A/B
        for (i, (path, full_filter)) in item.filters.into_iter().rev().enumerate() {
            result = result
                + &if i == 0 {
                    format!(
                        "\n# {} filters\n{}if",
                        item.name,
                        if !item.begin_with_else { "" } else { "els" }
                    )
                } else {
                    " elsif".to_string()
                }
                + " envelope :localpart :matches \"to\" "
                + &serde_json::to_string(&full_filter.localparts).unwrap()
                + " {"
                // Generate sieve code for labels 
                + &code_block({
                    let mut all_keywords = HashSet::new();
                    let silent = if full_filter.options.unwrap().silent.unwrap() {
                        "\naddflag \"\\\\Seen\";\nfileinto \"unread\";"
                    } else {
                        ""
                    }
                    .to_string();
                    const IF_HEADER_CONTAINS: &str = "\nif header :contains [\"from\",\"subject\"] ";
                    let labels = if let Some(full_filter_labels) = full_filter.labels {
                        /*
                        1 mail can have multiple labels, thus we cannot use if else but only if
                        if () {
                            fileinto label 1
                        }
                        if () {
                            fileinto label 2
                        }
                        else {
                            mark as seen
                        }
                        but we need an else to mark as seen if silent & not having any label
                        (by default label overwrite silent option)
                        this wouldn't work since else apply only to the last if
                        therefore we need this flag to know when to wrap those if in a big if
                        that contains all the keywords
                        */
                        let multiple_labels = full_filter_labels.len() > 1;
                        let mut labels = "".to_string();
                        for (label, keywords) in full_filter_labels.into_iter() {
                            labels = labels
                                + IF_HEADER_CONTAINS
                                + &serde_json::to_string(&keywords).unwrap()
                                + " {"
                                + &code_block(format!("\nfileinto \"{}\";", label))
                                + "\n}";
                            // If we indeed need to wrap the if then we need to store all the keywords
                            if !silent.is_empty() && multiple_labels {
                                keywords.iter().for_each(|keyword| {
                                    all_keywords.insert(keyword.clone());
                                })
                            }
                        }
                        labels
                    } else {
                        "".to_string()
                    };
                    // If not silent then just show all the if
                    if silent.is_empty() {
                        labels
                    // If silent then if no labels just show silent
                    } else if labels.is_empty() {
                        silent
                    // Else both are there
                    } else {
                        // There's all_keywords implies that there's multiple labels, need to wrap it
                        let mut all_keywords = all_keywords.drain().collect::<Vec<String>>();
                        all_keywords.sort();
                        (if !all_keywords.is_empty() {
                            IF_HEADER_CONTAINS.to_string()
                                + &serde_json::to_string(&all_keywords).unwrap()
                                + " {"
                                + &code_block(labels)
                                + "\n}"
                        } else {
                            labels
                        }) + " else {"
                            + &code_block(silent)
                            + "\n}"
                    }
                })
                ///////////////
                + &code_block({
                    let mut cumulated_path = "".to_string();
                    let mut file_into = "".to_string();
                    /*
                    Fileinto from parent to child, this allow the mail to fallback
                    to one of the parent folder in case the child doesn't exist
                    */
                    for folder in path.split('/').collect::<Vec<_>>() {
                        cumulated_path = if cumulated_path.is_empty() {
                            folder.to_string()
                        } else {
                            format!("{}/{}", cumulated_path, folder)
                        };
                        file_into = file_into
                            + &format!(
                                "\nfileinto \"{}{}\";",
                                item.domain_folder, cumulated_path
                            )
                    }
                    file_into
                })
                + "\n}";
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::types::FilterOptions;

    #[test]
    fn filter_generator() {
        let mut labels = super::BTreeMap::new();
        labels.insert(
            "label".to_string(),
            super::StringOrArray::Array(vec!["l".to_string()]),
        );
        let mut fg = super::FilterGenerator::new("Test", "@domain/".to_string(), false);
        fg.generate(
            "Home bills/zrocery",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec!["custom".to_string()]),
                labels: None,
                options: Some(FilterOptions {
                    generic: None,
                    orphan: None,
                    silent: Some(true),
                }),
            },
        )
        .generate(
            "Home bills/trocery",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec!["custom1".to_string()]),
                labels: Some(labels.clone()),
                options: Some(FilterOptions {
                    generic: None,
                    orphan: None,
                    silent: Some(true),
                }),
            },
        );
        labels.insert(
            "label1".to_string(),
            super::StringOrArray::Array(vec!["l".to_string(), "m".to_string()]),
        );
        fg.generate(
            "Home bills/grocery",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec![
                    "home-bills.grocery".to_string(),
                    "custom2".to_string(),
                ]),
                labels: Some(labels.clone()),
                options: Some(FilterOptions {
                    generic: None,
                    orphan: None,
                    silent: Some(true),
                }),
            },
        )
        .generate(
            "Home bills/frocery",
            super::FullFilter {
                localparts: super::StringOrArray::Array(vec!["custom3".to_string()]),
                labels: None,
                options: None,
            },
        );
        assert_eq!(
            fg.into_string_with_unknown(),
            r#"
# Test filters
if envelope :localpart :matches "to" ["custom"] {
    addflag "\\Seen";
    fileinto "unread";
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/zrocery";
} elsif envelope :localpart :matches "to" ["custom1"] {
    if header :contains ["from","subject"] ["l"] {
        fileinto "label";
    } else {
        addflag "\\Seen";
        fileinto "unread";
    }
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/trocery";
} elsif envelope :localpart :matches "to" ["home-bills.grocery","custom2"] {
    if header :contains ["from","subject"] ["l","m"] {
        if header :contains ["from","subject"] ["l"] {
            fileinto "label";
        }
        if header :contains ["from","subject"] ["l","m"] {
            fileinto "label1";
        }
    } else {
        addflag "\\Seen";
        fileinto "unread";
    }
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/grocery";
} elsif envelope :localpart :matches "to" ["custom3"] {
    fileinto "@domain/Home bills";
    fileinto "@domain/Home bills/frocery";
} else {
    addflag "\\Seen";
    fileinto "Unknown";
}"#
        );
    }

    #[test]
    #[should_panic(expected = "with empty folder path")]
    fn filter_generator_panic_empty_path() {
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false)
            .generate(
                "",
                super::FullFilter {
                    localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                    labels: None,
                    options: None,
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
                options: None,
            },
        );
        assert_eq!(Into::<String>::into(fg), "");
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
                    options: None,
                },
            );
    }

    #[test]
    #[should_panic(expected = "label keywords")]
    fn filter_generator_panic_label_keyword_empty() {
        let mut labels = super::BTreeMap::new();
        labels.insert(
            "label".to_string(),
            super::StringOrArray::String("".to_string()),
        );
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false)
            .generate(
                "path",
                super::FullFilter {
                    localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                    labels: Some(labels),
                    options: None,
                },
            );
    }

    #[test]
    #[should_panic(expected = "label keywords")]
    fn filter_generator_panic_label_keywords_empty() {
        let mut labels = super::BTreeMap::new();
        labels.insert(
            "label".to_string(),
            super::StringOrArray::Array(vec!["".to_string()]),
        );
        super::FilterGenerator::new("Test filter generator", "@domain/".to_string(), false)
            .generate(
                "path",
                super::FullFilter {
                    localparts: super::StringOrArray::Array(vec!["lp".to_string()]),
                    labels: Some(labels),
                    options: None,
                },
            );
    }
}
