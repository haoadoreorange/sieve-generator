mod common;
mod generators;
mod types;

use crate::{
    common::{code_block, GLOBAL_OPTIONS},
    generators::DomainGenerator,
    types::{PanicOnEmpty, Retirable, SieveDomainConfig, StringOrArray},
};
use clap::{crate_authors, crate_version, Arg};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

// enforce config json file structure { [domain: string]: JSON }
type SieveConfig = HashMap<String, HashMap<String, serde_json::Value>>;

fn read_sieve_config_json(file_path: &str) -> SieveConfig {
    File::open(file_path)
        .and_then(|file| -> Result<SieveConfig, std::io::Error> {
            serde_json::from_reader(BufReader::new(file)).map_err(|e| e.into())
        })
        .expect("No config json file could be opened. Abort mission, repeat, abort !")
}

fn prepare(
    mut sieve_domain_config: HashMap<String, serde_json::Value>,
) -> (SieveDomainConfig, Vec<String>, bool) {
    let mut secrets = vec!["pikachu".to_string()];
    let mut domain_as_first_folder = false;
    if let Some(options) = sieve_domain_config.remove("options") {
        match options {
            serde_json::Value::Object(mut options) => {
                if let Some(options_secrets) = options.remove("secrets") {
                    secrets = match serde_json::from_value::<StringOrArray>(options_secrets)
                        .unwrap()
                        .panic_on_empty("secrets")
                    {
                        StringOrArray::String(secret) => vec![secret],
                        StringOrArray::Array(options_secrets) => options_secrets,
                    };
                }
                if let Some(b) = options.remove("domain-as-first-folder") {
                    domain_as_first_folder = if let serde_json::Value::Bool(b) = b {
                        b
                    } else {
                        panic!("Dear my beloved user...domain-as-first-folder must be boolean !");
                    };
                }
            }
            _ => {
                panic!("Don't you know domain options must be object ((; ?");
            }
        };
    }
    (
        serde_json::from_value(serde_json::to_value(sieve_domain_config).unwrap()).unwrap(),
        secrets,
        domain_as_first_folder,
    )
}

fn main() {
    let matches = clap::App::new("Sieve filter generator")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Set config file path, default to sieve.config.json")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .value_name("FILE")
                .help("Set prefix sieve file path, default to prefix.sieve if exists")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Set output file path, default to filter.sieve")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("sieve.config.json");
    let prefix_path = matches.value_of("prefix").unwrap_or("prefix.sieve");
    let mut output_path = matches
        .value_of("output")
        .unwrap_or("filter.sieve")
        .to_string();
    if Path::new(&output_path).is_dir() {
        output_path = format!("{}/filter.sieve", output_path);
    }

    let sieve_config = read_sieve_config_json(config_path).into_iter();
    if sieve_config.len() > 1 {
        GLOBAL_OPTIONS.lock().unwrap().force_domain_as_first_folder = true;
    }

    let mut sieve_code = "".to_string();
    for (i, (domain, sieve_domain_config)) in sieve_config.enumerate() {
        let (sieve_domain_config, secrets, domain_as_first_folder) = prepare(sieve_domain_config);
        sieve_code = sieve_code
            + &format!("\n# @{}", domain)
            + if i == 0 { "\nif" } else { "\n elsif" }
            + &format!(" envelope :domain :is \"to\" \"{}\" {{", domain)
            + &code_block({
                let mut g = DomainGenerator::new(&domain, secrets, domain_as_first_folder);
                g.generate(sieve_domain_config);
                g.retire()
            })
            + "\n}";
    }
    let content = fs::read_to_string(prefix_path).unwrap_or_default() + &sieve_code;
    fs::write(&output_path, &content).unwrap_or_else(|_| {
        println!(
            "Write to {} failed, dumpling final content to stdout...\n{}",
            output_path, content
        )
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn prepare() {
        let (whitelist, secrets, domain_as_first_folder) = super::prepare(
            serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
                r#"
                {
                    "options": {
                        "domain-as-first-folder": true,
                        "secrets": ["something"]
                    }, 
                    "Newsletter": {
                        "Business": "wallstreet"
                    } 
                }"#,
            )
            .unwrap(),
        );
        assert_eq!(
            serde_json::to_string(&whitelist).unwrap(),
            "{\"Newsletter\":{\"Business\":\"wallstreet\"}}",
        );
        assert_eq!(secrets, vec!["something".to_string()]);
        assert!(domain_as_first_folder);
    }

    #[test]
    fn prepare_no_secrets() {
        let (whitelist, secrets, domain_as_first_folder) = super::prepare(
            serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
                r#"
                {
                    "options": {
                        "domain-as-first-folder": true
                    }, 
                    "Newsletter": {
                        "Business": "wallstreet"
                    } 
                }"#,
            )
            .unwrap(),
        );
        assert_eq!(
            serde_json::to_string(&whitelist).unwrap(),
            "{\"Newsletter\":{\"Business\":\"wallstreet\"}}",
        );
        assert_eq!(secrets, vec!["pikachu".to_string()]);
        assert!(domain_as_first_folder);
    }

    #[test]
    #[should_panic(expected = "must be object")]
    fn prepare_panic_options_not_object() {
        super::prepare(
            serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
                r#"
                {
                    "options": ""
                }"#,
            )
            .unwrap(),
        );
    }

    #[test]
    #[should_panic(expected = "must be boolean")]
    fn prepare_panic_options_first_folder_not_boolean() {
        super::prepare(
            serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
                r#"
                {
                    "options": {
                        "domain-as-first-folder": "",
                        "secrets": ["something"]
                    } 
                }"#,
            )
            .unwrap(),
        );
    }
}
