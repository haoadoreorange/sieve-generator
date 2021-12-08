use crate::types::{PanicOnEmpty, StringOrArray};
use lazy_static::lazy_static;
use std::sync::Mutex;

pub struct GlobalOptions {
    pub force_domain_as_first_folder: bool,
}
lazy_static! {
    pub static ref GLOBAL_OPTIONS: Mutex<GlobalOptions> = Mutex::new(GlobalOptions {
        force_domain_as_first_folder: false,
    });
}

pub fn code_block<T: AsRef<str>>(s: T) -> String {
    indentasy::indent(s, 1, 4)
}

impl StringOrArray {
    pub fn to_array(self) -> Vec<String> {
        // TODO: Add test
        match self {
            StringOrArray::String(s) => {
                vec![s]
            }
            StringOrArray::Array(array) => array,
        }
    }
}

impl AsRef<StringOrArray> for StringOrArray {
    fn as_ref(&self) -> &StringOrArray {
        self
    }
}

impl PanicOnEmpty for StringOrArray {
    fn panic_on_empty(self, variable_name: &str) -> Self {
        _panic_on_empty(self, variable_name)
    }
}

impl PanicOnEmpty for &StringOrArray {
    fn panic_on_empty(self, variable_name: &str) -> Self {
        _panic_on_empty(self, variable_name)
    }
}

fn _panic_on_empty<T: AsRef<StringOrArray>>(arg: T, variable_name: &str) -> T {
    match arg.as_ref() {
        StringOrArray::String(value) => {
            if value.is_empty() {
                panic!("{} cannot be empty string", variable_name);
            }
        }
        StringOrArray::Array(value) => {
            if value.is_empty() {
                panic!("Array of {} cannot be empty", variable_name);
            }
            for string in value {
                if string.is_empty() {
                    panic!("Array of {} cannot contain empty string", variable_name);
                }
            }
        }
    }
    arg
}

#[cfg(test)]
mod tests {
    use super::PanicOnEmpty;

    #[test]
    #[should_panic(expected = "cannot be empty")]
    fn panic_on_empty_string() {
        super::StringOrArray::String("".to_string()).panic_on_empty("test");
    }

    #[test]
    #[should_panic(expected = "cannot be empty")]
    fn panic_on_empty_array() {
        super::StringOrArray::Array(vec![]).panic_on_empty("test");
    }

    #[test]
    #[should_panic(expected = "cannot contain empty")]
    fn panic_on_array_empty_string() {
        (&super::StringOrArray::Array(vec!["".to_string()])).panic_on_empty("test");
    }
}
