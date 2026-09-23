# Changelog

All notable changes to this project will be documented in this file.

The format is based on "Keep a Changelog" (https://keepachangelog.com/) and this project adheres to Semantic Versioning (SemVer).

## Unreleased

### Added

- Expanded documentation: updated [README.md](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/README.md) with usage examples, API details, and testing instructions.
- Add this [CHANGELOG.md](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/CHANGELOG.md) to track releases and notable changes.

### Changed

- None yet.

### Fixed

- None yet.


## [0.1.0] - 2026-09-24

### Added

- Initial public release of `peisar_frontmatter`:
  - `parse_markdown_frontmatter<T>(content: &str) -> Result<ParseResult<T>, String>` — parses optional YAML front matter and returns a typed representation plus the Markdown body (see [src/lib.rs](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/src/lib.rs)).
  - `ParseResult<T>` type with convenience accessors: `yaml_data()` and `pure_markdown_content()`.
  - YAML deserialization backed by `serde_yaml` and generic over any `T: serde::de::DeserializeOwned`.
  - Unit tests covering front matter parsing, CRLF handling, and documents without front matter.
  - Minimal dependencies declared in [Cargo.toml](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/Cargo.toml).
- Add license file ([LICENSE](/home/ptm/Github/PEISAR/peisar-mono-repo/packages/peisar_frontmatter/LICENSE)).


## Notes on releasing

- Bump the crate version in the workspace or package manifest before tagging a release.
- Suggested tag name for this initial release: `v0.1.0`.


---

This file is intended to be updated on each release. Move Unreleased items into a new version section and add the release date when publishing.
