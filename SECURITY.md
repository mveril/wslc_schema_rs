# Security Policy

## Supported Versions

Security fixes are handled on the maintained `develop` and release branches. For published crates, report issues against the latest released version unless the issue exists only on an unreleased branch.

## Reporting a Vulnerability

Do not open a public issue for a suspected vulnerability. Report it through GitHub Security Advisories when available, or contact the maintainer listed in [Cargo.toml](Cargo.toml) privately.

Include the affected crate, generated artifact, build script, or workflow; reproduction steps; expected and actual behavior; and impact on parsing, schema validation, code generation, dependency supply chain, or publication.

## Scope

Pay particular attention to:

- Parsing untrusted or malformed `wslc_schema.h` input.
- Generated Rust or JSON Schema that changes serialization behavior.
- Checksums and provenance metadata under `wslc_schema_rs/generated/`.
- CI and release automation, including crates.io tokens and upstream source fetching.

Never commit registry tokens, credentials, or private upstream source material.

