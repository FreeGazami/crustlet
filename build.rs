use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct Bin {
    name: String,
}

// create enum to handle string or string array
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SignleOrArray {
    Single(String),
    Array(Vec<String>),
}

#[derive(Deserialize)]
struct Build {
    target: SignleOrArray,
}

fn creat_passoff_env() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    let manifest = fs::read_to_string(&cargo_toml_path)?;

    let cargo_toml: CargoToml = toml::from_str(&manifest).expect("Failed to parse Cargo.toml");

    let package_name = &cargo_toml.package.name;
    let package_version = &cargo_toml.package.version;

    let bin_name = match &cargo_toml.bin {
        Some(bins) if !bins.is_empty() => &bins[0].name,
        _ => package_name,
    };

    let env_string: String = format!(
        "PACKAGE_NAME={}\nPACKAGE_VERSION={}\nBIN_NAME={}\n",
        package_name, package_version, bin_name
    );
    let env_path = ".env";

    let mut file = File::create(env_path)?;
    file.write_all(env_string.as_bytes())
        .expect("Failed to write to pass over file to setup post build script environment");

    Ok(())
}

fn main() {
    creat_passoff_env().expect("Failed to create env file used to pass over variables");
}
