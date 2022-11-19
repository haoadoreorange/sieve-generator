use crate::types::{FilterOptions, PanicOnEmpty, StringOrArray};
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

impl From<StringOrArray> for Vec<String> {
    fn from(item: StringOrArray) -> Self {
        match item {
            StringOrArray::String(s) => {
                vec![s]
            }
            StringOrArray::Array(array) => array,
        }
    }
}

// Needed to use same function _panic_on_empty for both T and &T
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

/*
Since trait PanicOnEmpty consume self, there is no way rust can
coerce that to a reference type, thus we must explicitely implement
the trait for &type too
*/
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

impl FilterOptions {
    // If mask property is not None then take it
    pub fn mask_with(&self, mask: &Self) -> Self {
        let mut new = self.clone();
        if mask.generic.is_some() {
            new.generic = mask.generic;
        }
        if mask.orphan.is_some() {
            new.orphan = mask.orphan;
        }
        if mask.silent.is_some() {
            new.silent = mask.silent;
        }
        new
    }
}

#[cfg(test)]
mod tests {
    use crate::types::FilterOptions;

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

    #[test]
    fn string_to_array() {
        assert_eq!(
            vec![""],
            Vec::<String>::from(super::StringOrArray::String("".to_string()))
        );
    }

    #[test]
    fn mask() {
        assert_eq!(
            FilterOptions {
                generic: Some(true),
                orphan: Some(true),
                silent: Some(true)
            },
            FilterOptions {
                generic: None,
                orphan: Some(true),
                silent: Some(false)
            }
            .mask_with(&FilterOptions {
                generic: Some(true),
                orphan: None,
                silent: Some(true)
            })
        );
    }
}
