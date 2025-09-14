//! CREATE A DISK IMAGE
//!
//!

use std::env;
// use std::fs;
// use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;

fn make_disk(img: String) -> Result<ExitStatus, std::io::Error> {
    Command::new("qemu-img")
        .arg("create")
        .arg("-f")
        .arg("raw")
        .arg(img)
        .arg("64M")
        .status()
}

fn makefs(img: String) -> Result<ExitStatus, std::io::Error> {
    Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg(img)
        .status()
}

fn main() {
    let triple = env::var("TARGET").expect("TARGET env variable not set");
    let img_file = format!("crustlet-{}.img", triple);

    match make_disk(img_file.clone()) {
        Ok(_) => {
            println!("created img file");
        }
        Err(e) => {
            println!("error: {e:?}");
            return ();
        }
    }

    match makefs(img_file.clone()) {
        Ok(_) => {
            println!("created fat32 on img file");
        }
        Err(e) => {
            println!("error: {e:?}");
            return ();
        }
    }
}