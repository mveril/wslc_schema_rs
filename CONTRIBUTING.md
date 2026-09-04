# Contributing

Thanks for contributing to `wslc_schema_rs`. This workspace publishes Serde types generated from WSL's `wslc_schema.h`; keep changes focused, reproducible, and tied to a WSL source version where appropriate.

## Branches

Use feature and fix branches from `develop`, and target pull requests to `develop`. Prepare releases on `release/<crate-version>` (for example, `release/0.1.0+2.9.3`); `main` contains stable releases. Documentation or CI-only changes may target `main` when they do not affect published crate code.

## Development

Use the repository's Rust 2024 edition and formatting configuration. Before opening a pull request, run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo xtask check
cargo deny check
```

`cargo xtask generate` rewrites every artifact in `wslc_schema_rs/generated/`; do not edit those files manually. Use `cargo xtask generate --header PATH` for a local header. Run `cargo xtask check --upstream` only when network access is intended and the pinned WSL tag should be verified.

## Tests and Pull Requests

Add unit tests beside hand-written code and integration/fixture coverage in `wslc_schema_rs/tests/`. A schema change must update committed generated artifacts and ensure each generated definition has official WSL fixture coverage.

Follow the pull-request template. Include the upstream WSL tag, source header change, schema/API impact, and commands run. Keep commits focused and use short imperative subjects, such as `Update schema for WSL 2.9.4`.

## Security

Do not commit credentials, registry tokens, private keys, local headers containing non-public material, or machine-specific paths. Report suspected vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

