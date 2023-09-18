use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SieveDomainConfig {
    SimpleFilter(StringOrVec),
    FullFilter(FullFilter),
    SubDomainConfig(HashMap<String, SieveDomainConfig>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullFilter<T = StringOrVec, O = Option<FilterOptions>> {
    pub localparts: T,
    pub labels: Option<BTreeMap<String, T>>,
    pub options: O,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct FilterOptions<B = Option<bool>> {
    pub generic: B,  // Generate generic filter.
    pub fullpath: B, // If generic, use full path (w parent prefix) in localpart.
    #[serde(alias = "mark-as-read")]
    pub mark_as_read: B,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

// use lazy_static::lazy_static;
// use std::sync::Mutex;
// pub struct GlobalOptions {
//     pub force_domain_as_first_folder: bool,
// }
// lazy_static! {
//     pub static ref GLOBAL_OPTIONS: Mutex<GlobalOptions> = Mutex::new(GlobalOptions {
//         force_domain_as_first_folder: false,
//     });
// }

pub fn code_block<T: AsRef<str>>(s: T) -> String {
    indentasy::indent(s, 1, 4)
}

impl From<StringOrVec> for Vec<String> {
    fn from(o: StringOrVec) -> Self {
        match o {
            StringOrVec::String(s) => vec![s],
            StringOrVec::Vec(v) => v,
        }
    }
}

impl StringOrVec {
    pub fn panic_on_empty(self, variable_name: &str) -> Self {
        match &self {
            StringOrVec::String(string) => {
                if string.is_empty() {
                    panic!("{} cannot be empty string.", variable_name);
                }
            }
            StringOrVec::Vec(vec) => {
                if vec.is_empty() {
                    panic!("Array of {} cannot be empty.", variable_name);
                }
                for string in vec {
                    if string.is_empty() {
                        panic!("Array of {} cannot contain empty string.", variable_name);
                    }
                }
            }
        }
        self
    }
}

impl FilterOptions {
    pub fn unwrap_or_default(&self, default: FilterOptions<bool>) -> FilterOptions<bool> {
        let mut new = default;
        if let Some(v) = self.generic {
            new.generic = v;
        }
        if let Some(v) = self.fullpath {
            new.fullpath = v;
        }
        if let Some(v) = self.mark_as_read {
            new.mark_as_read = v;
        }
        new
    }
}

pub fn is_unknown(path: &str) -> bool {
    Regex::new(r"^Unknown").unwrap().is_match(path)
}

// #[cfg(test)]
// mod tests {
//     use crate::types::FilterOptions;

//     #[test]
//     #[should_panic(expected = "cannot be empty")]
//     fn panic_on_empty_string() {
//         super::StringOrVec::String("".to_string()).panic_on_empty("test");
//     }

//     #[test]
//     #[should_panic(expected = "cannot be empty")]
//     fn panic_on_empty_array() {
//         super::StringOrVec::Vec(vec![]).panic_on_empty("test");
//     }

//     // #[test]
//     // #[should_panic(expected = "cannot contain empty")]
//     // fn panic_on_array_empty_string() {
//     //     (&super::StringOrArray::Array(vec!["".to_string()])).panic_on_empty("test");
//     // }

//     #[test]
//     fn string_to_array() {
//         assert_eq!(
//             vec![""],
//             Vec::<String>::from(super::StringOrVec::String("".to_string()))
//         );
//     }

//     // #[test]
//     // fn mask() {
//     //     assert_eq!(
//     //         FilterOptions {
//     //             generic: Some(true),
//     //             fullpath: Some(true),
//     //             mark_as_read: Some(true)
//     //         },
//     //         FilterOptions {
//     //             generic: None,
//     //             fullpath: Some(true),
//     //             mark_as_read: Some(false)
//     //         }
//     //         .mask_with(&FilterOptions {
//     //             generic: Some(true),
//     //             fullpath: None,
//     //             mark_as_read: Some(true)
//     //         })
//     //     );
//     // }
// }
