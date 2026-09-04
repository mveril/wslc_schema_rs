mod model;
mod parser;
mod rustgen;
mod schema;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use git2::{FetchOptions, Oid, Repository};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const WSL_REPOSITORY: &str = "https://github.com/microsoft/WSL.git";
const HEADER_PATH: &str = "src/windows/inc/wslc_schema.h";
const SCHEMA_FILE: &str = "wslc_schema.schema.json";
const RUST_FILE: &str = "wslc_schema.rs";
const METADATA_FILE: &str = "metadata.json";
const CHECKSUM_FILE: &str = "checksum.sha256";

#[derive(Parser)]
#[command(author, version, about = "Generate wslc_schema_rs from WSL")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate and write every committed artifact.
    Generate {
        /// Read a local header rather than fetching the matching WSL tag.
        #[arg(long)]
        header: Option<PathBuf>,
    },
    /// Verify committed artifacts without changing them.
    Check {
        /// Also fetch and compare the header from the matching WSL tag.
        #[arg(long)]
        upstream: bool,
    },
}

#[derive(Debug)]
struct Source {
    contents: String,
    repository: String,
    tag: String,
    path: String,
    commit_oid: Option<String>,
    blob_oid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GenerationMetadata {
    source_repository: String,
    source_tag: String,
    source_path: String,
    source_commit_oid: Option<String>,
    source_blob_oid: Option<String>,
    source_sha256: String,
    generator_version: String,
    typify_version: String,
    schema_draft: String,
    schema_file: String,
    rust_file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace = MetadataCommand::new().no_deps().exec()?;
    let workspace_root = workspace.workspace_root.as_std_path();
    let crate_package = workspace
        .workspace_packages()
        .into_iter()
        .find(|package| package.name.as_str() == "wslc_schema_rs")
        .context("workspace package wslc_schema_rs not found")?;
    let wsl_version = crate_package.version.build.as_str();
    if wsl_version.is_empty() {
        bail!("wslc_schema_rs version must contain WSL build metadata (for example +2.9.3)");
    }
    let output_dir = workspace_root.join("wslc_schema_rs").join("generated");

    match cli.command {
        Command::Generate { header } => {
            let source = match header {
                Some(path) => local_source(&path, wsl_version)?,
                None => git_source(wsl_version)?,
            };
            let artifacts = generate(&source)?;
            write_artifacts(&output_dir, &artifacts)?;
            println!("generated WSL schema artifacts for {wsl_version}");
        }
        Command::Check { upstream } => {
            check_committed(&output_dir, wsl_version, upstream)?;
            println!("generated artifacts are current and valid");
        }
    }
    Ok(())
}

struct Artifacts {
    schema: String,
    rust: String,
    metadata: String,
}

fn generate(source: &Source) -> Result<Artifacts> {
    let declarations = parser::parse(&source.contents)?;
    let schema_value = schema::generate(&declarations, &source.tag)?;
    let mut schema = serde_json::to_string_pretty(&schema_value)?;
    schema.push('\n');
    let rust = rustgen::generate(&schema)?;
    let metadata = GenerationMetadata {
        source_repository: source.repository.clone(),
        source_tag: source.tag.clone(),
        source_path: source.path.clone(),
        source_commit_oid: source.commit_oid.clone(),
        source_blob_oid: source.blob_oid.clone(),
        source_sha256: sha256(source.contents.as_bytes()),
        generator_version: env!("CARGO_PKG_VERSION").to_owned(),
        typify_version: "0.6.2".to_owned(),
        schema_draft: "https://json-schema.org/draft/2020-12/schema".to_owned(),
        schema_file: SCHEMA_FILE.to_owned(),
        rust_file: RUST_FILE.to_owned(),
    };
    let mut metadata = serde_json::to_string_pretty(&metadata)?;
    metadata.push('\n');
    Ok(Artifacts {
        schema,
        rust,
        metadata,
    })
}

fn write_artifacts(output: &Path, artifacts: &Artifacts) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::write(output.join(SCHEMA_FILE), &artifacts.schema)?;
    fs::write(output.join(RUST_FILE), &artifacts.rust)?;
    fs::write(output.join(METADATA_FILE), &artifacts.metadata)?;
    let checksums = checksums(artifacts);
    fs::write(output.join(CHECKSUM_FILE), checksums)?;
    Ok(())
}

fn check_committed(output: &Path, version: &str, upstream: bool) -> Result<()> {
    let schema_text = fs::read_to_string(output.join(SCHEMA_FILE))?;
    let expected_rust = rustgen::generate(&schema_text)?;
    compare(output.join(RUST_FILE), &expected_rust)?;

    let metadata_text = fs::read_to_string(output.join(METADATA_FILE))?;
    let metadata: GenerationMetadata = serde_json::from_str(&metadata_text)?;
    if metadata.source_tag != version {
        bail!(
            "metadata tag {} does not match crate version {version}",
            metadata.source_tag
        );
    }
    let artifacts = Artifacts {
        schema: schema_text,
        rust: expected_rust,
        metadata: metadata_text,
    };
    compare(output.join(CHECKSUM_FILE), &checksums(&artifacts))?;

    if upstream {
        let source = git_source(version)?;
        let regenerated = generate(&source)?;
        compare(output.join(SCHEMA_FILE), &regenerated.schema)?;
        compare(output.join(RUST_FILE), &regenerated.rust)?;
        compare(output.join(METADATA_FILE), &regenerated.metadata)?;
        compare(output.join(CHECKSUM_FILE), &checksums(&regenerated))?;
    }
    Ok(())
}

fn compare(path: PathBuf, expected: &str) -> Result<()> {
    let actual =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if actual != expected {
        bail!("{} is stale; run `cargo xtask generate`", path.display());
    }
    Ok(())
}

fn checksums(artifacts: &Artifacts) -> String {
    format!(
        "{}  {}\n{}  {}\n{}  {}\n",
        sha256(artifacts.schema.as_bytes()),
        SCHEMA_FILE,
        sha256(artifacts.rust.as_bytes()),
        RUST_FILE,
        sha256(artifacts.metadata.as_bytes()),
        METADATA_FILE,
    )
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn local_source(path: &Path, version: &str) -> Result<Source> {
    Ok(Source {
        contents: fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?,
        repository: WSL_REPOSITORY.to_owned(),
        tag: version.to_owned(),
        path: HEADER_PATH.to_owned(),
        commit_oid: None,
        blob_oid: None,
    })
}

fn git_source(version: &str) -> Result<Source> {
    let temp = tempfile::tempdir()?;
    let repo = Repository::init_bare(temp.path())?;
    let mut remote = repo.remote_anonymous(WSL_REPOSITORY)?;
    let refspec = format!("+refs/tags/{version}:refs/tags/{version}");
    let mut fetch_options = FetchOptions::new();
    fetch_options.depth(1);
    remote.fetch(&[&refspec], Some(&mut fetch_options), None)?;

    read_header_blob(&repo, version)
}

fn read_header_blob(repo: &Repository, version: &str) -> Result<Source> {
    let object = repo.revparse_single(&format!("refs/tags/{version}"))?;
    let commit = object.peel_to_commit()?;
    let entry = commit.tree()?.get_path(Path::new(HEADER_PATH))?;
    let blob_oid: Oid = entry.id();
    let blob = repo.find_blob(blob_oid)?;
    let contents = std::str::from_utf8(blob.content())?.to_owned();
    Ok(Source {
        contents,
        repository: WSL_REPOSITORY.to_owned(),
        tag: version.to_owned(),
        path: HEADER_PATH.to_owned(),
        commit_oid: Some(commit.id().to_string()),
        blob_oid: Some(blob_oid.to_string()),
    })
}
