# wslc_schema_rs

`wslc_schema_rs` provides Serde models generated from Microsoft WSL's
`wslc_schema.h`. Rust field names are idiomatic, while serialized JSON retains
the original WSL property names.

The crate version's build metadata identifies the corresponding WSL release.
Generated source, JSON Schema, provenance metadata, and checksums are bundled
under `generated/`, so consuming the crate does not require network access.

For usage examples, development commands, regeneration, release policy, and
contribution guidance, see the [workspace README](../README.md).
