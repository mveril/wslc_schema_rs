use std::{collections::BTreeMap, env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Metadata {
    source_tag: String,
    schema_file: String,
    rust_file: String,
}

fn main() -> Result<()> {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").context("missing manifest dir")?);
    let generated = root.join("generated");
    let metadata_path = generated.join("metadata.json");
    let checksum_path = generated.join("checksum.sha256");
    let metadata: Metadata = serde_json::from_slice(&fs::read(&metadata_path)?)?;
    let package_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    if metadata.source_tag != package_version.build.as_str() {
        bail!(
            "generated WSL tag {} does not match package build metadata {}",
            metadata.source_tag,
            package_version.build
        );
    }

    let expected = parse_checksums(&fs::read_to_string(&checksum_path)?)?;
    let files = [
        &metadata.schema_file,
        &metadata.rust_file,
        &"metadata.json".to_owned(),
    ];
    for file in files {
        let path = generated.join(file);
        let actual = hex::encode(Sha256::digest(fs::read(&path)?));
        let wanted = expected
            .get(file.as_str())
            .with_context(|| format!("missing checksum for {file}"))?;
        if &actual != wanted {
            bail!(
                "checksum mismatch for {}; run `cargo xtask generate`",
                path.display()
            );
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed={}", checksum_path.display());
    Ok(())
}

fn parse_checksums(contents: &str) -> Result<BTreeMap<String, String>> {
    contents
        .lines()
        .map(|line| {
            let (hash, file) = line
                .split_once("  ")
                .with_context(|| format!("invalid checksum line: {line}"))?;
            Ok((file.to_owned(), hash.to_owned()))
        })
        .collect()
}
