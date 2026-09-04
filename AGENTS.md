# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 Cargo workspace. The public crate lives in `wslc_schema_rs/`; its hand-written entry point is `src/lib.rs`, while generated Rust, JSON Schema, metadata, and checksums are committed under `generated/`. Integration tests are in `wslc_schema_rs/tests/`, with captured WSL output in `tests/fixtures/wslc-<version>/`. The private `xtask/` crate parses Microsoft’s `wslc_schema.h` and regenerates the committed artifacts. Keep generator logic in `xtask/src/`; do not hand-edit files in `wslc_schema_rs/generated/`.

## Build, Test, and Development Commands

- `cargo build` builds the default `wslc_schema_rs` member offline from committed artifacts.
- `cargo test --workspace` runs unit and integration tests across both workspace crates.
- `cargo fmt --all -- --check` verifies standard Rust formatting.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` treats lint warnings as failures.
- `cargo xtask check` validates generated Rust, metadata, and checksums without network access.
- `cargo xtask generate` regenerates artifacts from the WSL tag encoded in the crate version; it normally requires network access. Use `--header PATH` for a local header.
- `cargo xtask check --upstream` fetches the pinned upstream tag and verifies exact reproducibility.
- `cargo deny check` audits advisories, licenses, dependency bans, and sources as configured in `deny.toml`.

## Coding Style & Naming Conventions

Use `rustfmt` defaults (four-space indentation) and idiomatic Rust naming: `snake_case` for modules, functions, and fields; `UpperCamelCase` for types; `SCREAMING_SNAKE_CASE` for constants. Keep code free of `unsafe`; the library enforces `#![forbid(unsafe_code)]`. Preserve upstream WSL JSON property names through Serde while exposing idiomatic Rust field names.

## Testing Guidelines

Use Rust’s built-in `#[test]` framework and descriptive `snake_case` test names. Add focused unit tests near hand-written behavior and end-to-end schema/fixture checks under `wslc_schema_rs/tests/`. When adding a generated schema definition, update official fixture coverage so every `$defs` entry is exercised and its Rust type deserializes successfully.

## Commit & Pull Request Guidelines

Git history is not included in this checkout, so use concise, imperative commit subjects such as `Add volume schema fixture`. Keep generator changes and regenerated artifacts in the same commit. Follow `.github/PULL_REQUEST_TEMPLATE.md`; explain the source WSL version, summarize schema/API impact, and list validation commands run. Release branches use `release/<crate-version>` and tags use `v<crate-version>`, including WSL build metadata (for example, `release/0.1.0+2.9.3`).

## Generated Artifacts & Versioning

The crate version’s build metadata identifies the matching WSL release (for example, `0.1.0+2.9.3`). If that version changes, regenerate all artifacts and commit `wslc_schema.schema.json`, `wslc_schema.rs`, `metadata.json`, and `checksum.sha256` together.
