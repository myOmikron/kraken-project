use std::{env, fs};

use probe_config::schema::ProbeFile;

fn main() {
    let Some(file) = env::args().nth(1) else {
        println!("This is a small program to verify the syntax of our custom .probe format. Please pass a file to check as argument.");
        return;
    };
    let content = fs::read_to_string(&file).expect("Failed to read file");
    let service = serde_yaml::from_str::<ProbeFile>(&content).unwrap();
    println!("{service:#?}");
}
