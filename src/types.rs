use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullFilter<T> {
    pub localparts: T,
    pub labels: Option<BTreeMap<String, T>>,
    pub silent: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SieveDomainConfig {
    SimpleFilter(StringOrArray),
    FullFilter(FullFilter<StringOrArray>),
    Object(HashMap<String, SieveDomainConfig>),
    Boolean(bool),
}

pub trait PanicOnEmpty {
    fn panic_on_empty(self, name: &str) -> Self;
}

pub trait Retirable {
    fn retire(self) -> String;
}
