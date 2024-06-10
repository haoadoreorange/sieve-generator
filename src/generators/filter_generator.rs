use crate::common::{code_block, is_unknown, FilterOptions, FullFilter, StringOrVec};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct FilterGenerator<'a> {
    name: &'a str, // Name of the generator
    domain_folder: String,
    filters: BTreeMap<String, FullFilter<Vec<String>, FilterOptions<bool>>>,
    begin_with_else: bool, // It can begin with else if generated after another.
}

impl<'a> FilterGenerator<'a> {
    //
    pub fn new(name: &'a str, domain_folder: String, begin_with_else: bool) -> FilterGenerator<'a> {
        FilterGenerator {
            name,
            domain_folder,
            filters: BTreeMap::new(),
            begin_with_else,
        }
    }

    pub fn generate(
        &mut self,
        path: &str,
        full_filter: FullFilter<StringOrVec, FilterOptions<bool>>,
    ) -> &mut Self {
        //
        if path.is_empty() {
            panic!(
                "ERROR: calling {}.generate() with empty folder path, something is wrong.",
                self.name
            );
        }
        if let StringOrVec::String(localpart) = &full_filter.localparts {
            if localpart.is_empty() {
                return self; // Filters localpart can take an empty string to use only generic.
            }
        }

        /* Convert StringOrVec to Vec before insert for easier to_string(). */
        let localparts: Vec<String> = full_filter.localparts.panic_on_empty("localparts").into();
        let labels = if let Some(full_filter_labels) = full_filter.labels {
            let mut labels = BTreeMap::new();
            for (label, keywords) in full_filter_labels.into_iter() {
                StringOrVec::String(label.clone()).panic_on_empty("label");
                labels.insert(
                    label,
                    Vec::<String>::from(keywords.panic_on_empty("label keywords")),
                );
            }
            Some(labels)
        } else {
            None
        };
        self.filters.insert(
            path.to_string(),
            FullFilter::<Vec<String>, FilterOptions<bool>> {
                localparts,
                labels,
                options: full_filter.options,
            },
        );

        self
    }

    pub fn to_string_with_unknown(&self) -> String {
        self.to_string()
            + " else {"
            + &code_block("\naddflag \"\\\\Seen\";\nfileinto \"Unknown\";")
            + "\n}"
    }
}

impl Display for FilterGenerator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = "".to_string();
        /* rev() is for generic filter, A/B must be filtered before A, otherwise a.b will all go to A, not A/B */
        for (i, (path, full_filter)) in self.filters.iter().rev().enumerate() {
            result = result
                + &if i == 0 {
                    format!(
                        "\n# {} filters\n{}if",
                        self.name,
                        if !self.begin_with_else { "" } else { "els" }
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
                    /*
                     * Fileinto from parent to child, this allow the mail to fallback
                     * to one of the parent folder in case the child doesn't exist
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
                                self.domain_folder, cumulated_path
                            )
                    }
                    file_into
                })

                /* Generate sieve code for labels. */
                + &code_block({
                    let mut all_keywords = HashSet::new();
                    let mark_as_read = if full_filter.options.mark_as_read {
                        "\naddflag \"\\\\Seen\";".to_string() +
                        if !is_unknown(&path) {
                            "\nfileinto \"unread\";"
                        } else {
                            ""
                        }
                    } else {
                        "".to_string()
                    };
                    const IF_HEADER_CONTAINS: &str = "\nif header :contains [\"from\",\"subject\"] ";
                    let labels = if let Some(full_filter_labels) = &full_filter.labels {
                        /*
                         * 1 mail can have multiple labels, thus we cannot use if else but only if
                         * ```
                         * if () {
                         *     fileinto label 1
                         * }
                         * if () {
                         *     fileinto label 2
                         * }
                         * else {
                         *     mark as seen
                         * }
                         * ```
                         * but we need an else to mark as read if no label condition is met (by default label overwrite mark-as-read option). This wouldn't
                         * work since else apply only to the last if. Therefore we need
                         * this flag to know when to wrap those if in a big if that contains
                         * all the keywords.
                        */
                        let multiple_labels = full_filter_labels.len() > 1;
                        let mut labels = "".to_string();
                        for (label, keywords) in full_filter_labels.iter() {
                            labels = labels
                                + IF_HEADER_CONTAINS
                                + &serde_json::to_string(&keywords).unwrap()
                                + " {"
                                + &code_block(format!("\nfileinto \"{}\";", label))
                                + "\n}";
                            /* If we indeed need to wrap the if then we need to store all the keywords. */
                            if !mark_as_read.is_empty() && multiple_labels {
                                keywords.iter().for_each(|keyword| {
                                    all_keywords.insert(keyword.clone());
                                })
                            }
                        }
                        labels
                    } else {
                        "".to_string()
                    };
                    /* If not mark-as-read then just show all the if. */
                    if mark_as_read.is_empty() {
                        labels
                    /* Else then if no labels just show mark as read. */
                    } else if labels.is_empty() {
                        mark_as_read
                    /* Else both are there. */
                    } else {
                        /* There's all_keywords implies that there's multiple labels, need to wrap it. */
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
                            + &code_block(mark_as_read)
                            + "\n}"
                    }
                })
                + "\n}";
        }
        write!(f, "{}", result)
    }
}
