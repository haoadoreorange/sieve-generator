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

impl PanicOnEmpty for StringOrArray {
    fn panic_on_empty(self, name: &str) -> Self {
        match self {
            StringOrArray::String(ref value) => {
                if value.is_empty() {
                    panic!("{} cannot be empty string", name)
                }
            }
            StringOrArray::Array(ref value) => {
                if value.is_empty() {
                    panic!("Array of {} cannot be empty", name)
                }
                for secret in value {
                    if secret.is_empty() {
                        panic!("Array of {} cannot contain empty string", name)
                    }
                }
            }
        }
        self
    }
}

pub fn panic_on_empty<T: AsRef<Vec<String>>>(arg: T, variable_name: &str) -> T {
    if arg.as_ref().is_empty() {
        panic!("Array of {} cannot be empty", variable_name)
    }
    for string in arg.as_ref() {
        if string.is_empty() {
            panic!("Array of {} cannot contain empty string", variable_name)
        }
    }
    arg
}

impl PanicOnEmpty for &StringOrArray {
    fn panic_on_empty(self, variable_name: &str) -> Self {
        match self {
            StringOrArray::String(ref value) => {
                if value.is_empty() {
                    panic!("{} cannot be empty string", variable_name);
                }
            }
            StringOrArray::Array(ref value) => {
                panic_on_empty(value, variable_name);
            }
        }
        self
    }
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
