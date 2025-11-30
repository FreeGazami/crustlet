#![no_std]

mod configs;

use alloc::collections::BTreeMap;
use uefi::Status;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;

// parse
pub fn parse_env(bytes: Vec<u8>) -> BTreeMap<String, String> {
    let mut configs: BTreeMap<String, String> = BTreeMap::new();
    let mut slice: &[u8] = bytes.as_slice();

    while let Some(index) = slice.iter().position(|&b| b == '\n') {
        let line_slice = &slice[..index];

        let clean_line = if line_slice.ends_with(b"\r") {
            &line_slice[..line_slice.len() - 1]
        } else {
            line_slice
        };

        let line = match str::from_utf8(clean_line) {
            Ok(line) => {
                line
            },
            Err(e) => {
                slice = &slice[index + 1..];
                continue;
            },
        };

        // process line
        if let Some(index) = line.bytes().position(|item| item == b'=') {
            if index != (line.len() - 1) && index != 0 {
                configs.insert(line[..index].to_string(), line[index+1..].to_string()); 
            } 
        }

        slice = &slice[index + 1..];
    }

    configs
}

