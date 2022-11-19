use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FilterOptions {
    pub orphan: Option<bool>,  // No need to add parent prefix in localpart
    pub generic: Option<bool>, // Not generate generic filter
    pub silent: Option<bool>,  // Mark as seen
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullFilter<T> {
    pub localparts: T,
    pub labels: Option<BTreeMap<String, T>>,
    pub options: Option<FilterOptions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

// Same function to check if string or array or string in array empty for StringOrArray
pub trait PanicOnEmpty {
    fn panic_on_empty(self, name: &str) -> Self;
}

// Represent recursive JSON for domain config
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SieveDomainConfig {
    SimpleFilter(StringOrArray),
    FullFilter(FullFilter<StringOrArray>),
    SubDomainConfig(HashMap<String, SieveDomainConfig>),
}
