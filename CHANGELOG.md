# Changelog

All notable changes to the Grove element are documented in this file. Format:
[Keep a Changelog](https://keepachangelog.com/).

Grove is the WASM extension sandbox - an isolated runtime for executing VS Code
extensions in WebAssembly with WASI, providing secure boundaries between
extension code and the host editor.

## [v2.1] - Q2 2026: DevLog + Refinement

### Changed

- Replaced `tracing` crate with custom `DevLog!` macro for tag-filtered logging
  across all modules (consistent with Mountain/Air/Cocoon convention)
- Rust formatting standardized across all 68 .rs source files
- Documentation and table formatting refined in `DeepDive.md`

## [v2.0] - Q1 2026: Module Expansion

### Added

- Comprehensive sandbox implementation with 68 Rust modules
- `Proto/` directory with gRPC service definition for Mountain ↔ Grove
  communication
- PascalCase naming convention enforced throughout source tree
- Comprehensive README with architecture tables and module breakdown
- CI/CD workflows (`.github/workflows/`) and Dependabot configuration

## [v1.2] - Q3-Q4 2025: Foundation Build

### Added

- Initial WASM runtime scaffolding with `build.rs` for WASI target configuration
- Cargo workspace integration as `Grove` member of Land root workspace
- Documentation infrastructure (`Documentation/`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`)
- GitHub Actions integration for Rust builds

## [v1.1] - Q2 2025: Project Inception

### Added

- Repository created April 2025 as part of NLnet NGI0 Commons Fund initiative
- Initial README with architecture planning
- Register as CodeEditorLand/Grove GitHub repository
- License (CC0) + CODE_OF_CONDUCT.md
