use std::{env, fs};

use probe_config::parse::{ParseError, Service};

fn main() -> Result<(), ParseError> {
    let Some(file) = env::args().nth(1) else {
        println!("This is a small program to verify the syntax of our custom .probe format. Please pass a file to check as argument.");
        return Ok(());
    };
    let content = fs::read_to_string(&file).expect("Failed to read file");
    Service::from_file(&file, &content).map(|_| ())
}
