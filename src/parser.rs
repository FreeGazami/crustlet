#![no_std]

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

mod build_conf;

fn main() -> io::Result<()> {
    const ENV_FILE: &str = build_conf::ENV_FILE;

    let file = File::open(ENV_FILE)?;
    let reader = BufReader::new(file);
    let mut configs: HashMap<String, String> = HashMap::new();

    for line in reader.lines() {
        let line = line?;

        // let parts: Vec<&str> = line.split("=").collect();
        let mut iter = line.split("=");

        // println!("{}", line);
        // println!("{:?}", parts);

        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            configs.entry(key.to_string()).or_insert(value.to_string());
        } else {
            eprintln!("Skipping malformed line: {}", line);
        }
    }

    println!("{:?}", configs);

    Ok(())
}
