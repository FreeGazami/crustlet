use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
struct CargoToml {
    package: Package,
    bin: Option<Vec<Bin>>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct BuildData {
    package_name: String,
    package_version: String,
    bin_name: String,
    triple: String,
}

#[derive(Deserialize)]
struct Bin {
    name: String,
}

fn creat_passoff_json() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    let manifest = fs::read_to_string(&cargo_toml_path)?;

    let cargo_toml: CargoToml = toml::from_str(&manifest).expect("Failed to parse Cargo.toml");

    let package_name = &cargo_toml.package.name;
    let package_version = &cargo_toml.package.version;

    let bin_name = match &cargo_toml.bin {
        Some(bins) if !bins.is_empty() => &bins[0].name,
        _ => package_name,
    };

    let triple = env::var("TARGET").expect("Tareget not set");

    let build_data: BuildData = BuildData { 
        package_name: package_name.to_string(),
        package_version: package_version.to_string(),
        bin_name: bin_name.to_string(),
        triple: triple.to_string(),
     };

    let json_string = serde_json::to_string(&build_data).expect("Serialization fail");

    let mut file = File::create("crustlet-build.json").expect("Failed to create json file");
    file.write_all(json_string.as_bytes()).expect("Failed to write to json file");

    Ok(())
}

fn main() {
    creat_passoff_json().expect("Failed to create env file used to pass over variables");
}
