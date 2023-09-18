#![doc = include_str!("../README.md")]

mod common;
mod generators;

use crate::{
    common::{code_block, SieveDomainConfig},
    generators::DomainGenerator,
};
use clap::Parser;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /*
     * Set config file path, default to sieve.config.json
     */
    #[arg(short, long, value_name = "FILE", default_value = "sieve.config.json")]
    config: String,

    /*
     * Set prefix sieve file path, default to prefix.sieve if exists.
     */
    #[arg(short, long, value_name = "FILE", default_value = "prefix.sieve")]
    prefix: String,

    /*
     * Set output file path, default to filter.sieve
     */
    #[arg(short, long, value_name = "FILE", default_value = "filter.sieve")]
    output: String,
}

fn main() {
    let mut args = Args::parse();
    if Path::new(&args.output).is_dir() {
        args.output = format!("{}/filter.sieve", args.output);
    }

    // let sieve_config = read_sieve_config_json(&args.config).into_iter();
    // If there is >1 domain we must distinguish the folders by a domain parent folder
    // if sieve_config.len() > 1 {
    //     GLOBAL_OPTIONS.lock().unwrap().force_domain_as_first_folder = true;
    // }

    let mut sieve_code = "".to_string();
    for (i, (domain, sieve_domain_config)) in
        read_sieve_config_json(&args.config).into_iter().enumerate()
    {
        let (sieve_domain_config, domain_as_first_folder) = prepare(sieve_domain_config);
        sieve_code = sieve_code
            + &format!("\n# @{domain}")
            + if i == 0 { "\nif" } else { " elsif" }
            + &format!(" envelope :domain :is \"to\" \"{domain}\" {{")
            + &code_block({
                let mut g = DomainGenerator::new(
                    &domain,
                    domain_as_first_folder, // || GLOBAL_OPTIONS.lock().unwrap().force_domain_as_first_folder,
                );
                g.generate(sieve_domain_config);
                String::from(g)
            })
            + "\n}";
    }
    let content = fs::read_to_string(&args.prefix).unwrap_or_default() + &sieve_code; // Prepend the prefix sieve and write to output
    fs::write(&args.output, &content).unwrap_or_else(|_| {
        println!(
            "Write to {} failed, dumpling final content to stdout...\n{}",
            args.output, content
        )
    });
}

/*
 * Read as type SieveConfig to avoid having to include the global options in the type
 * since there's only one and it will be removed from the object anyway.
 */
/*
 * Enforce config json file structure.
 * {
 *     [domain: string]: {
 *          options,
 *         [dirname: string]: JSON
 *    }
 * }
 */
type SieveConfig = HashMap<String, HashMap<String, serde_json::Value>>;
fn read_sieve_config_json(file_path: &str) -> SieveConfig {
    File::open(file_path)
        .and_then(|file| -> Result<SieveConfig, std::io::Error> {
            serde_json::from_reader(BufReader::new(file)).map_err(|e| e.into())
        })
        .unwrap()
}

/*
 * Take out the options object and return the domain config object.
 */
fn prepare(
    mut sieve_domain_config: HashMap<String, serde_json::Value>,
) -> (SieveDomainConfig, bool) {
    let mut domain_as_first_folder = false;
    if let Some(options) = sieve_domain_config.remove("options") {
        match options {
            serde_json::Value::Object(mut options) => {
                if let Some(value) = options.remove("domain-as-first-folder") {
                    domain_as_first_folder = serde_json::from_value::<bool>(value)
                        .expect("Haizaa... domain-as-first-folder must be bool...");
                }
            }
            _ => {
                panic!("Don't you know domain options must be object ((; ?");
            }
        };
    }
    (
        serde_json::from_value(serde_json::to_value(sieve_domain_config).unwrap()).unwrap(),
        domain_as_first_folder,
    )
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn prepare() {
//         let (whitelist, domain_as_first_folder) = super::prepare(
//             serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
//                 r#"
//                 {
//                     "options": {
//                         "domain-as-first-folder": true
//                     },
//                     "Newsletter": {
//                         "Business": "wallstreet"
//                     }
//                 }"#,
//             )
//             .unwrap(),
//         );
//         assert_eq!(
//             serde_json::to_string(&whitelist).unwrap(),
//             "{\"Newsletter\":{\"Business\":\"wallstreet\"}}",
//         );
//         assert!(domain_as_first_folder);
//     }

//     #[test]
//     #[should_panic(expected = "must be object")]
//     fn prepare_panic_options_not_object() {
//         super::prepare(
//             serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
//                 r#"
//                 {
//                     "options": ""
//                 }"#,
//             )
//             .unwrap(),
//         );
//     }

//     #[test]
//     #[should_panic(expected = "expected a boolean")]
//     fn prepare_panic_options_first_folder_not_boolean() {
//         super::prepare(
//             serde_json::from_str::<super::HashMap<String, serde_json::Value>>(
//                 r#"
//                 {
//                     "options": {
//                         "domain-as-first-folder": ""
//                     }
//                 }"#,
//             )
//             .unwrap(),
//         );
//     }
// }
