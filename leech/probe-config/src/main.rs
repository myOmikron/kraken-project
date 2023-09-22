use std::env;
use std::fs;

use probe_config::{ParseError, Service};

fn main() -> Result<(), ()> {
    let Some(file) = env::args().skip(1).next() else {
        println!("This is a small program to verify the syntax of our custom .probe format. Please pass a file to check as argument.");
        return Err(());
    };
    let file = fs::read_to_string(&file).expect("Failed to read file");
    if let Err(err) = Service::from_file(&file) {
        match err {
            ParseError::MissingService => {
                println!("The file should start with `service: <name>`");
            }
            ParseError::MissingPrevalence => {
                println!("The `service: <name>` line should be followed by `prevalence: <often|average|obscure>`");
            }
            ParseError::MissingProbes => {
                println!("Missing list of probes `probes:\\n  - <probe declaration>");
            }
            ParseError::DuplicateValue(value, line) => {
                println!("The value {value} in line {line} has already been set");
            }
            ParseError::MissingValue(value, probe) => {
                println!("The probe started in line {probe} is missing the value {value}");
            }
            ParseError::UnknownValue(line) => {
                println!("Unknown value in line {line}");
            }
            ParseError::ConflictingPayload { probe_line } => {
                println!("The probe started in line {probe_line} has two conflicting payloads");
            }
            ParseError::ValueAfterSubRegex(line) => {
                println!("There is a value after a `sub_regex` list in line {line}");
            }
            ParseError::UnexpectedSubRegex(line) => {
                println!("`sub_regex` item outside of list in line {line}");
            }
            ParseError::MissingSubRegex { probe_line } => {
                println!(
                    "The probe started in line {probe_line} has an empty `sub_regex` list \
                    (If you don't want any, then remove the list completely)"
                );
            }
            ParseError::InvalidProtocol(line) => {
                println!("Invalid protocol in line {line}");
            }
            ParseError::InvalidPrevalence(line) => {
                println!("Invalid prevalence in line {line}");
            }
        }
        Err(())
    } else {
        Ok(())
    }
}
