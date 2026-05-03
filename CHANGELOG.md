# Changelog - Grove

Grove is our Rust/WASM extension-host sidecar - the future home for extensions
we want to run sandboxed in WebAssembly instead of the Node.js Cocoon process.
This file records what we built in our voice, version by version. Format adapted
from [Keep a Changelog](https://keepachangelog.com/).

## [v2.1] - Full Workbench Lift (April 2026)

We took Grove from "scaffold + WASM runtime" to "actually forwards
language-provider registrations to Mountain over the same transport the rest of
the fleet uses."

### Added

- **Language-provider registration system** (`dfa6dad`, 2026-04-05) - Grove can
  now accept `register*Provider` calls from a hosted WASM extension and hold the
  provider registry in process.
- **Provider-registration forwarding to Mountain via transport** (`cb994d4`,
  2026-04-06). The registry from `dfa6dad` now round-trips so the workbench sees
  Grove's provider list the same way it sees Cocoon's.
- **Comprehensive README expansion** (`ef66c60`, 2026-04-05) - first pass at
  user-facing documentation now that the host actually does something.

### Changed

- **Custom DevLog tag-filtered logging** replaces `tracing` across the crate
  (`dd8750f`, 2026-04-16). Brings Grove in line with the `LAND_DEV_LOG` knob the
  rest of the fleet honours.
- **Module files renamed from snake_case to PascalCase** (`3305757`,
  2026-04-17). Brought Grove in line with the project-wide naming rule.
- **Re-exports removed**, module-prefix usage enforced (`564fc2a`, 2026-04-18).
  Call sites now spell the full reverse-hierarchical path so it's obvious which
  file owns a symbol.
- **`Grove.proto` field names migrated to PascalCase** (`8eeb25e`, 2026-04-11) -
  the gRPC contract now matches the Rust side.
- **Transport-layer PascalCase migration** completed (`5b390de`, `c10204d`,
  2026-04-04), obsolete snake_case stubs removed (`c3a3f0a`).
- **`TransportConfig` from Common library properly stored in
  `TransportAdapter`** (`5b5f89d`, 2026-04-06) - Grove now consumes Common's
  transport vocabulary instead of carrying its own.
- **`LAND_DEV_LOG` env var renamed to `Trace`** (`0f25ffc`, 2026-04-28) for the
  user-facing knob.

### Fixed

- **String ownership in `LanguageNamespace` provider registration** (`fbe3523`,
  2026-04-06).

## [v2.0] - Editor Launch (Q1 2026: Scaffold → Working Host)

The buildout cycle. Grove went from empty repository to a working Rust/WASM
extension host with VS Code API facade, gRPC transport, and a passing
integration-test suite, all inside February 2026.

### Added (February 2026 - Initial Scaffold)

- **Rust/WASM extension-host scaffold** (`a408d36`, 2026-02-08).
- **Binary runtime, CLI, and Mountain service registration** (`ca02de2`,
  2026-02-08) - the sidecar can now be spawned by Mountain the same way Cocoon
  is.
- **Common module**: core error types, shared traits, utility modules
  (`219323e`, 2026-02-08).
- **VS Code API facade and core types** (`14b552b`, 2026-02-08).
- **Host orchestration, activation, and lifecycle management** (`0804b36`,
  2026-02-08).
- **Application-level gRPC contract** + configuration service (`0e4bf8c`,
  2026-02-08).
- **Modular transport layer** with gRPC, IPC, and WASM strategies (`d2e1132`,
  2026-02-08).
- **WebAssembly runtime engine and execution environment** (`e49e368`,
  2026-02-08).
- **Comprehensive integration test suite** for the extension host (`15ba658`,
  2026-02-08).

### Changed (February 2026)

- **Wasmtime 20 integration stabilised**, naming collisions resolved (`78ca1c4`,
  2026-02-10).
- **Host function execution implemented**, API surface stabilised (`46218c8`,
  2026-02-10).
- **Spine action client introduced**, formatting standardised, dependencies
  upgraded (`0764347`, 2026-02-09).
- **WASM error handling refined**, Rust deps updated (`a3d3f8a`, 2026-02-27).
- **Unused-variable warnings and dead code cleared** (`88e037d`, 2026-02-27).
- **Workbench fix** (`133eeea`, 2026-02-14) - the consumer side in Mountain
  learned to talk to Grove cleanly during the editor-launch boot path.

### Added / Changed (March 2026)

- **VS Code API namespaces renamed for consistency** (`2784988`, 2026-03-03).
- **Workspace dependencies standardised** across the crate (`b026f4c`,
  2026-03-04).
- **Project-wide naming-convention standardisation** (`e93aa69`, 2026-03-04).
- **Common Transport Adapter** implemented to bridge Grove transports through
  Common's vocabulary (`6f5d8a1`, 2026-03-13).
- **Consistent code formatting and indentation** sweep (`43b0109`, 2026-03-11).

## [v1.x] - Pre-Scaffold (April - September 2025)

Grove existed as a placeholder repository through 2025. Four commits across the
year (`7987ab6` 2025-04-16, `e9371dd` 2025-06-02, `4a4656e` 2025-06-08,
`98c62c5` 2025-09-09) - all unlabelled scaffolding pushes. No published source
surface. The real implementation arrived in v2.0 with the February 2026 buildout
above.

## [v0.0] - Project Inception

Empty repository created. Grove sat in the architecture diagrams as "the future
Rust/WASM extension host" while we built out Cocoon (the Node.js extension host)
and the rest of the fleet. The 14-commit buildout on **2026-02-08** (`a408d36`
through `15ba658`) was the first real work.
