# **Grove**&#x2001;🌳

<table>
	<tr>
		<td>
			<a href="https://GitHub.Com/CodeEditorLand/Grove" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/last-commit/CodeEditorLand/Grove?label=Update&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/last-commit/CodeEditorLand/Grove?label=Update&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/last-commit/CodeEditorLand/Grove?label=Update&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Update" title="Update" />
				</picture>
			</a>
			<br />
			<a href="https://GitHub.Com/CodeEditorLand/Grove" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/issues/CodeEditorLand/Grove?label=Issue&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/issues/CodeEditorLand/Grove?label=Issue&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/issues/CodeEditorLand/Grove?label=Issue&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Issue" title="Issue" />
				</picture>
			</a>
		</td>
		<td>
			<a href="https://github.com/CodeEditorLand/Grove" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/stars/CodeEditorLand/Grove?style=flat&label=Star&logo=github&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/stars/CodeEditorLand/Grove?style=flat&label=Star&logo=github&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/stars/CodeEditorLand/Grove?style=flat&label=Star&logo=github&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Star" title="Star" />
				</picture>
			</a>
			<br />
			<a href="https://GitHub.Com/CodeEditorLand/Grove" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/downloads/CodeEditorLand/Grove/total?label=Download&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/downloads/CodeEditorLand/Grove/total?label=Download&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/downloads/CodeEditorLand/Grove/total?label=Download&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Download" title="Download" />
				</picture>
			</a>
		</td>
	</tr>
</table>

The Native `Rust`/`WebAssembly` Extension Host for Land&#x2001;🏞️

> **VS Code extensions run with full `Node.js` capabilities in a shared process.
> A malicious or buggy extension can access any file, make any network request,
> and read another extension's state. The extension sandbox is a policy
> document, not a technical boundary.**

_"An extension can only touch what you explicitly grant. The sandbox is enforced
by the hardware, not a policy."_

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Grove/blob/Current/LICENSE)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Rust Version](https://img.shields.io/badge/Rust-1.95.0+-orange.svg)](https://www.rust-lang.org/)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Rust Edition](https://img.shields.io/badge/Edition-2024-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-WASM-blue.svg)](https://webassembly.org/) [![WASMtime Version](https://img.shields.io/badge/WASMtime-v45-blue.svg)](https://wasmtime.dev/)

**[Rust API Documentation](https://rust.documentation.grove.editor.land/)**&#x2001;📖

---

## Overview

**Grove**&#x2001;🌳 is the high-performance `Rust`/`WebAssembly` extension host
for the **Land**&#x2001;🏞️ Code Editor. It complements `Cocoon`&#x2001;🦋
(`Node.js`) by providing a native environment for running `Rust` and
`WASM`-compiled VS Code extensions. Grove offers secure sandboxing through
`WASMtime`, multiple transport strategies (`gRPC`, `IPC`, `WASM`), and full
compatibility with the VS Code API surface.

VS Code extensions run with full `Node.js` capabilities in a shared process - a
malicious or buggy extension can access any file, make any network request, and
read another extension's state. Grove solves this by enforcing sandboxing at the
hardware level: an extension can only touch what you explicitly grant.

**Grove is engineered to:**

1. **Provide Native Extension Hosting** - Execute `Rust` extensions with zero
   overhead through static linking or `WASM` sandboxing.
2. **Enable Secure Sandboxing** - Isolate untrusted extensions using
   `WASMtime`'s capability-based security model with configurable memory limits
   and resource controls.
3. **Support Multiple Transports** - Communicate with `Mountain`&#x2001;⛰️ via
   `gRPC`, `IPC`, or direct `WASM` host function calls for flexible deployment.
4. **Maintain Cocoon Compatibility** - Share the same VS Code API surface and
   activation semantics for seamless extension porting between the `Node.js` and
   native hosting environments.

---

## Key Features&#x2001;🔐

**`WASM` Runtime Integration** - Full `WebAssembly` support through `WASMtime`,
with capability-based security, configurable memory limits, `WASMtime` fuel
metering to bound execution time, and explicit host-function grants. Extensions
are sandboxed at the hardware level.

**Multiple Transport Strategies** - A strategy-pattern transport layer
supporting `gRPC` (to `Mountain`&#x2001;⛰️'s `Vine`&#x2001;🌿 server on port
50051), `IPC` (Unix socket for local communication), `WASM` (direct host
function calls), and `Mist`&#x2001;🌫️ (message-bus integration with the `Mist`
pub/sub system, behind the `websocket` Cargo feature).

**Standalone Operation** - Can run as an independent process with its own
lifecycle, or connect to `Mountain`&#x2001;⛰️ via `gRPC` for distributed
deployment. The `Transport/CommonAdapter` unifies all transport backends behind
a single interface.

**Cross-Platform** - Native support for `macOS`, `Linux`, and `Windows` with
platform-specific optimizations. The `Binary/Main/` entry point handles platform
signal handling and daemon initialization.

**`VS Code` API Compatibility** - Implements `vscode.d.ts` type definitions
through the `APIBridge` facade, with `API/VSCode/` providing typed wrappers for
the full VS Code extension API surface including commands, windows, and
notifications.

**Secure by Default** - `#![deny(unsafe_code)]` at the crate level, `WASMtime`
capability-based isolation, configurable memory limits per extension, and
explicit host-function grants ensure no extension can escape its sandbox.

---

## Core Architecture Principles&#x2001;🏗️

| Principle                 | Description                                                                                                                                    | Key Components                                                                                                                                             |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Security First**        | Isolate extensions via `WASMtime`'s capability model with configurable resource limits. Unsafe code is denied at the crate level.              | `WASM/Runtime`, `WASM/MemoryManager`, `WASM/HostBridge`                                                                                                    |
| **Transport Agnosticism** | Multiple communication strategies behind a unified `Transport/Strategy` trait so deployment choice is a config flag, not a code change.        | `Transport/Strategy`, `Transport/CommonAdapter`, `Transport/gRPCTransport`, `Transport/IPCTransport`, `Transport/WASMTransport`, `Transport/MistTransport` |
| **API Surface Parity**    | Implement the full VS Code extension API (`vscode.d.ts`) so extensions port seamlessly between `Cocoon`&#x2001;🦋 and `Grove`&#x2001;🌳.       | `API/VSCode`, `API/Types`, `Host/APIBridge`, `Host/Activation`                                                                                             |
| **Composability**         | Modular separation of host core, `WASM` runtime, transport layer, and protocol handling. Each module can be compiled and tested independently. | `Host/*`, `WASM/*`, `Transport/*`, `Protocol/*`, `API/*`                                                                                                   |

---

## System Architecture

```mermaid
graph LR
    classDef grove    fill:#d0d8ff,stroke:#4a6fa5,stroke-width:2px,color:#001050;
    classDef mountain fill:#f0d0ff,stroke:#9b59b6,stroke-width:2px,color:#2c0050;
    classDef wasm     fill:#d4f5d4,stroke:#27ae60,stroke-width:2px,color:#0a3a0a;
    classDef transport fill:#fff3c0,stroke:#f39c12,stroke-width:1px,stroke-dasharray:5 5,color:#5a3e00;
    classDef cocoon   fill:#cce8ff,stroke:#2980b9,stroke-width:1px,stroke-dasharray:5 5,color:#003050;

    subgraph GROVE["Grove 🌳 - Rust/WASM Extension Host"]
        direction TB
        subgraph HOST["Host/ - Extension Lifecycle"]
            ExtHost["ExtensionHost.rs 🏡 main controller"]:::grove
            ExtMgr["ExtensionManager.rs 🔍 discovery + loading"]:::grove
            Activation["Activation/ ⚡ activation events"]:::grove
            Lifecycle["Lifecycle.rs"]:::grove
            APIBridge["APIBridge.rs 🌉 vscode.d.ts facade"]:::grove
            ExtHost --> ExtMgr --> Activation --> Lifecycle
            Activation --> APIBridge
        end
        subgraph API["API/ - VS Code API Surface"]
            VSCode["VSCode/ 📋 typed API wrappers"]:::grove
            Types["Types.rs 🧱 shared type definitions"]:::grove
            APIBridge --> VSCode --> Types
        end
        subgraph WASM_RT["WASM/ - WASMtime Runtime"]
            WASMRuntime["Runtime/ 🚀 WASMtime engine + store"]:::wasm
            ModLoader["ModuleLoader/ 📦 compile + instantiate"]:::wasm
            MemMgr["MemoryManager/ 📏 allocation + limits"]:::wasm
            HostBridge["HostBridge/ 🔗 host↔WASM calls"]:::wasm
            WASMRuntime --> ModLoader
            ModLoader --> MemMgr
            WASMRuntime --> HostBridge
        end
        subgraph TRANSPORT["Transport/ - Strategy Pattern"]
            Strategy["Strategy.rs - trait"]:::transport
            CommonAdapter["CommonAdapter.rs 🔌 unified backend"]:::transport
            gRPC["gRPCTransport.rs"]:::transport
            IPC["IPCTransport.rs"]:::transport
            WASMTrans["WASMTransport.rs"]:::transport
            MistTrans["MistTransport.rs 💬 pub/sub bus"]:::transport
            Strategy --- CommonAdapter
            CommonAdapter --- gRPC
            CommonAdapter --- IPC
            CommonAdapter --- WASMTrans
            CommonAdapter --- MistTrans
        end
        subgraph PROTO["Protocol/"]
            SpineConn["SpineConnection.rs 🦴 Spine protocol"]:::grove
            SpineAction["SpineActionClient/ 🎬 action dispatch"]:::grove
            SpineConn --> SpineAction
        end

        APIBridge --> WASMRuntime
        HostBridge --> Strategy
        SpineConn --> gRPC
    end

    subgraph MOUNTAIN["Mountain ⛰️"]
        VineGRPC["Vine gRPC Server 🌿"]:::mountain
    end

    subgraph COCOON["Cocoon 🦋 complementary host"]
        CocoonRef["Node.js extension host same vscode API surface"]:::cocoon
    end

    gRPC -- gRPC :50051 --> VineGRPC
    IPC -- Unix socket --> VineGRPC
    MistTrans -. message bus, websocket feature .-> VineGRPC
    APIBridge -.shares API surface.-> CocoonRef
```

**Connection paths:**

| Path                                            | Protocol                                  | Use Case                                                 |
| ----------------------------------------------- | ----------------------------------------- | -------------------------------------------------------- |
| Grove&#x2001;🌳 → Mountain&#x2001;⛰️ via `gRPC` | Protobuf over `gRPC` on port 50051        | Distributed deployment, remote extensions                |
| Grove → Mountain via `IPC`                      | Unix domain socket                        | Local single-machine communication                       |
| Grove → Mountain via `Mist`&#x2001;🌫️           | Message-bus pub/sub (`websocket` feature) | Event-driven, decoupled workflows                        |
| Grove → Cocoon&#x2001;🦋                        | Shared API surface                        | Extension portability between native and `Node.js` hosts |
| Extension → `WASMtime`                          | `WASM` host functions                     | Sandboxed extension execution                            |
| `APIBridge` → `API/VSCode`                      | Direct call                               | Typed VS Code API wrappers                               |

---

## Key Components

| Component             | Path                                      | Description                                                                                                                                                          |
| --------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ExtensionHost         | `Source/Host/ExtensionHost.rs`            | Main controller (`ExtensionHostImpl`) managing the full extension lifecycle                                                                                          |
| ExtensionManager      | `Source/Host/ExtensionManager.rs`         | Extension discovery, validation, and loading                                                                                                                         |
| Activation            | `Source/Host/Activation/`                 | Activation events and contribution point handling (`ActivationEngine`, `ActivationEvent`, `ActivationHandler`, `ActivationContext`, `ActivationRecord`, `WildMatch`) |
| Lifecycle             | `Source/Host/Lifecycle.rs`                | Extension state machine (install, enable, disable, uninstall)                                                                                                        |
| APIBridge             | `Source/Host/APIBridge.rs`                | VS Code API facade implementing `vscode.d.ts`                                                                                                                        |
| VSCode API            | `Source/API/VSCode/`                      | Typed wrappers for the full VS Code extension API surface (`VSCodeAPI`, `Window`, `Workspace`, `Env`, `CommandNamespace`, `LanguageNamespace`, and more)             |
| API Types             | `Source/API/Types.rs`                     | Shared type definitions for extension API interactions                                                                                                               |
| WASM Runtime          | `Source/WASM/Runtime.rs`                  | `WASMtime` engine and store lifecycle                                                                                                                                |
| ModuleLoader          | `Source/WASM/ModuleLoader.rs`             | `WASM` module compilation and instantiation                                                                                                                          |
| MemoryManager         | `Source/WASM/MemoryManager.rs`            | Configurable memory limits and allocation tracking                                                                                                                   |
| HostBridge            | `Source/WASM/HostBridge.rs`               | Host-to-`WASM` function call dispatch                                                                                                                                |
| FunctionExport        | `Source/WASM/FunctionExport/`             | Export host functions to `WASM` guest modules (`HostFunctionRegistry`, `FunctionExportImpl`, `FunctionStats`)                                                        |
| Transport Strategy    | `Source/Transport/Strategy.rs`            | Transport strategy trait definition                                                                                                                                  |
| CommonAdapter         | `Source/Transport/CommonAdapter.rs`       | Unified transport backend routing                                                                                                                                    |
| gRPC Transport        | `Source/Transport/gRPCTransport.rs`       | `gRPC`-based communication with `Mountain`                                                                                                                           |
| IPC Transport         | `Source/Transport/IPCTransport.rs`        | Inter-process communication (Unix socket)                                                                                                                            |
| WASM Transport        | `Source/Transport/WASMTransport.rs`       | Direct `WASM` host-function communication                                                                                                                            |
| Mist Transport        | `Source/Transport/MistTransport.rs`       | Message-bus integration with `Mist` pub/sub (behind the `websocket` Cargo feature)                                                                                   |
| Spine Connection      | `Source/Protocol/SpineConnection.rs`      | `Spine` protocol client connection                                                                                                                                   |
| Spine Action Client   | `Source/Protocol/SpineActionClient/`      | Action dispatch over `Spine` protocol (`SpineActionClient`, `ReconnectStrategy`, `ConnectionStatus`)                                                                 |
| Configuration Service | `Source/Services/ConfigurationService.rs` | Service for managing extension-level configuration                                                                                                                   |
| Common Traits         | `Source/Common/Traits.rs`                 | Shared trait definitions for the extension host                                                                                                                      |
| Common Error          | `Source/Common/Error.rs`                  | Unified error types for the host layer                                                                                                                               |
| Runtime Build         | `Source/Binary/Build/RuntimeBuild.rs`     | Build-time runtime configuration                                                                                                                                     |
| Service Register      | `Source/Binary/Build/ServiceRegister.rs`  | Service registration at build time                                                                                                                                   |
| Entry                 | `Source/Binary/Main/Entry.rs`             | Platform entry point and daemon initialization                                                                                                                       |

---

## Project Structure&#x2001;🗺️

```
Element/Grove/
├── Source/
│   ├── Library.rs                     # Library root (cdylib + rlib)
│   ├── main.rs                        # Binary entry point
│   ├── DevLog.rs                      # Development logging infrastructure
│   ├── API/                           # VS Code API surface
│   │   ├── mod.rs                     # Module re-exports
│   │   ├── Types.rs                   # Shared API type definitions
│   │   └── VSCode/                    # Typed VS Code extension API wrappers
│   │       ├── mod.rs
│   │       ├── VSCodeAPI.rs           # Aggregate API entry point
│   │       ├── Window.rs              # vscode.window namespace
│   │       ├── Workspace.rs           # vscode.workspace namespace
│   │       ├── WorkspaceConfiguration.rs
│   │       ├── CommandNamespace.rs    # vscode.commands namespace
│   │       ├── LanguageNamespace.rs   # vscode.languages namespace
│   │       ├── ExtensionNamespace.rs  # vscode.extensions namespace
│   │       ├── Env.rs                 # vscode.env namespace
│   │       ├── CompletionItemProvider.rs
│   │       ├── DiagnosticCollection.rs
│   │       ├── DocumentSelector.rs
│   │       ├── Disposable.rs
│   │       ├── OutputChannel.rs
│   │       ├── ProviderStore.rs
│   │       └── Tests.rs
│   ├── Binary/                        # Binary initialization
│   │   ├── mod.rs
│   │   ├── Build/                     # Build-time configuration
│   │   │   ├── mod.rs
│   │   │   ├── RuntimeBuild.rs        # Build-time runtime configuration
│   │   │   └── ServiceRegister.rs     # Service registration at build time
│   │   └── Main/                      # Main entry point + platform init
│   │       ├── mod.rs
│   │       └── Entry.rs               # Platform entry point and daemon init
│   ├── Common/                        # Shared traits and error types
│   │   ├── mod.rs
│   │   ├── Traits.rs                  # Core trait definitions
│   │   └── Error.rs                   # Unified error types
│   ├── Host/                          # Extension lifecycle management
│   │   ├── mod.rs
│   │   ├── ExtensionHost.rs           # Main host controller (ExtensionHostImpl)
│   │   ├── ExtensionManager.rs        # Discovery and loading
│   │   ├── Lifecycle.rs               # Lifecycle state machine
│   │   ├── APIBridge.rs               # VS Code API facade
│   │   └── Activation/                # Activation events
│   │       ├── mod.rs
│   │       ├── ActivationEngine.rs    # Activation event dispatcher
│   │       ├── ActivationEvent.rs     # Activation event types
│   │       ├── ActivationHandler.rs   # Per-event activation handlers
│   │       ├── ActivationContext.rs   # Activation context state
│   │       ├── ActivationRecord.rs    # Activation history record
│   │       ├── WildMatch.rs           # Glob-style pattern matching
│   │       └── Tests.rs
│   ├── Services/                      # Extension-level services
│   │   ├── mod.rs
│   │   └── ConfigurationService.rs    # Extension configuration management
│   ├── WASM/                          # WebAssembly runtime integration
│   │   ├── mod.rs
│   │   ├── Runtime.rs                 # WASMtime engine and store
│   │   ├── ModuleLoader.rs            # Module compilation + instantiation
│   │   ├── MemoryManager.rs           # Memory allocation and limits
│   │   ├── HostBridge.rs              # Host-to-WASM function calls
│   │   └── FunctionExport/            # Host function export to WASM
│   │       ├── mod.rs
│   │       ├── Default.rs
│   │       ├── ExportConfig.rs
│   │       ├── FunctionExportImpl.rs
│   │       ├── FunctionStats.rs
│   │       └── HostFunctionRegistry.rs
│   ├── Transport/                     # Communication strategies
│   │   ├── mod.rs
│   │   ├── Strategy.rs                # Transport strategy trait
│   │   ├── CommonAdapter.rs           # Unified transport backend
│   │   ├── TransportConfig.rs         # Shared transport configuration
│   │   ├── TransportFactory.rs        # Transport construction helpers
│   │   ├── gRPCTransport.rs           # gRPC to Mountain
│   │   ├── IPCTransport.rs            # Inter-process (Unix only)
│   │   ├── WASMTransport.rs           # Direct WASM communication
│   │   └── MistTransport.rs           # Mist message-bus integration (behind `websocket` feature)
│   └── Protocol/                      # Protocol handling
│       ├── mod.rs
│       ├── Constants.rs               # Protocol constants
│       ├── MessageType.rs             # Wire message type definitions
│       ├── ProtocolConfig.rs          # Protocol configuration
│       ├── ProtocolError.rs           # Protocol error types
│       ├── SpineConnection.rs         # Spine protocol client
│       ├── SpineActionClient/         # Action dispatch over Spine protocol
│       │   ├── mod.rs
│       │   ├── SpineActionClient.rs
│       │   ├── SpineConfig.rs
│       │   ├── ReconnectStrategy.rs
│       │   ├── ConnectionStatus.rs
│       │   ├── GroveCapabilities.rs
│       │   ├── HostInfo.rs
│       │   ├── calculate_backoff.rs
│       │   └── Tests.rs
│       └── Generated/                 # Code-generated protocol types
│           └── grove.rs               # Generated gRPC service definitions
├── Documentation/
│   └── Rust/
│       └── doc/                       # Cargo doc output
├── Cargo.toml
└── LICENSE
```

---

## In the Land Project

Grove serves as the native `Rust`/`WASM` extension host alongside
`Cocoon`&#x2001;🦋 (the `Node.js` host). Together they provide the two execution
environments for the Land&#x2001;🏞️ editor's extension model:

| Host                 | Language                   | Runtime                   | Sandboxing                             |
| -------------------- | -------------------------- | ------------------------- | -------------------------------------- |
| **Grove**&#x2001;🌳  | `Rust`, `WASM`             | `WASMtime`                | Hardware-enforced via capability model |
| **Cocoon**&#x2001;🦋 | `TypeScript`, `JavaScript` | `Node.js` via `Effect-TS` | Fiber-level process isolation          |

Grove communicates with `Mountain`&#x2001;⛰️ via `gRPC` (port 50051), `IPC`
(Unix socket), or the `Mist`&#x2001;🌫️ message bus for event-driven workflows.
It shares the same VS Code API surface as `Cocoon`, enabling seamless porting of
extensions between the `Node.js` and native hosting environments.

The `Transport/CommonAdapter` abstracts all communication strategies behind a
single interface, allowing deployment flexibility - standalone process,
distributed via `gRPC`, or integrated with `Mountain`'s `Vine` server.

---

## Getting Started&#x2001;🚀

### Prerequisites

- **Rust** 1.95.0 or later (edition 2024)
- Protocol Buffer compiler (optional, for proto file modifications)
- For `WASM` builds: `rustup target add wasm32-wasi`

### Build for Native

```bash
cd Element/Grove
cargo build --release
```

### Build for WASM

```bash
cd Element/Grove
cargo build --target wasm32-wasi --release
```

### Build with Features

```bash
# All features enabled
cargo build --release --features all

# WASM only
cargo build --release --features wasm

# gRPC only
cargo build --release --features grpc
```

### Available Features

| Feature     | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `default`   | Enables `grpc` and `wasm`                                    |
| `grpc`      | `gRPC` transport support (`tonic` + `prost`)                 |
| `wasm`      | `WebAssembly` runtime support (`wasmtime` + `wasmtime-wasi`) |
| `websocket` | `Mist` pub/sub `WebSocket` transport (`MistTransport`)       |
| `telemetry` | `OpenTelemetry` + `Prometheus` metrics export                |
| `dev`       | Enables `telemetry` for local development builds             |
| `debug`     | Enables `dev`                                                |
| `all`       | `grpc` + `wasm` + `telemetry`                                |

### As a Library

```rust
use Grove::{Host::ExtensionHost::ExtensionHostImpl, Transport::Strategy::Transport};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Host = ExtensionHostImpl::new(Transport::default()).await?;
    let ExtensionId = Host.load_extension(&"/path/to/extension".into()).await?;
    Host.activate(&ExtensionId).await?;
    Ok(())
}
```

---

## Security&#x2001;🔒

Grove enforces security at multiple layers:

| Layer              | Mechanism                                                                          |
| ------------------ | ---------------------------------------------------------------------------------- |
| **Crate level**    | `#![deny(unsafe_code)]` - no unsafe code permitted                                 |
| **Runtime**        | `WASMtime` capability-based isolation - each extension gets an independent sandbox |
| **Memory**         | Configurable per-extension memory limits via `WASM/MemoryManager`                  |
| **Resources**      | `WASMtime` fuel metering bounds per-extension execution time (`WASM/Runtime`)      |
| **Host functions** | Explicit capability grants - extensions must declare required host functions       |
| **Type safety**    | Full `Rust` type system across the host-WASM boundary                              |

---

## Compatibility

Grove is designed to be compatible with:

| Target                 | Integration                                                                      |
| ---------------------- | -------------------------------------------------------------------------------- |
| **Cocoon**&#x2001;🦋   | Shares VS Code API surface, activation semantics, and manifest parsing           |
| **VS Code**            | Implements `vscode.d.ts` type definitions                                        |
| **Mountain**&#x2001;⛰️ | Integrates via the `GroveService` `gRPC` protocol defined in `Proto/Grove.proto` |
| **Mist**&#x2001;🌫️     | Connects via `MistTransport` for event-driven pub/sub workflows                  |

---

## API Reference

- **[Rust API Documentation](https://rust.documentation.grove.editor.land/)**&#x2001;📖

---

## Related Documentation

- [Architecture Overview](https://Editor.Land/Doc/architecture) - Land&#x2001;🏞️
  system architecture
- [Why WebAssembly](https://Editor.Land/Doc/why-wasm) - Why `WASM` for extension
  sandboxing
- [Mountain](https://github.com/CodeEditorLand/Mountain)&#x2001;⛰️ - Native
  desktop shell and `gRPC` backend
- [Cocoon](https://github.com/CodeEditorLand/Cocoon)&#x2001;🦋 -
  `Node.js`/`Effect-TS` extension host
- [Mist](https://github.com/CodeEditorLand/Mist)&#x2001;🌫️ - Pub/sub message bus
  for event-driven workflows

---

## License&#x2001;⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license. You are free to use, modify, distribute, and build upon
this work for any purpose, without any restrictions. For the full legal text,
see the
[`LICENSE`](https://github.com/CodeEditorLand/Grove/tree/Current/LICENSE) file.

---

## Changelog&#x2001;📜

See
[`CHANGELOG.md`](https://github.com/CodeEditorLand/Grove/tree/Current/CHANGELOG.md)
for a history of changes specific to **Grove**&#x2001;🌳.

---

## Funding & Acknowledgements&#x2001;🙏🏻

This project is funded through
[NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
Next Generation Internet program, under grant agreement No 101135429.

The project is operated by PlayForm, based in Sofia, Bulgaria. PlayForm acts as
the open-source steward for Code Editor Land under the NGI0 Commons Fund grant.

<table>
	<tbody>
		<tr>
			<td align="left" valign="middle"><a href="https://Editor.Land"><img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land" /></a></td>
			<td align="left" valign="middle"><a href="https://PlayForm.Cloud"><img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm" /></a></td>
			<td align="left" valign="middle"><a href="https://NLnet.NL"><img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet" /></a></td>
			<td align="left" valign="middle"><a href="https://NLnet.NL/commonsfund"><img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund" /></a></td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@editor.land](mailto:Source/Open@editor.land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Grove) |
[Report an Issue](https://github.com/CodeEditorLand/Grove/issues) |
[Security Policy](https://github.com/CodeEditorLand/Grove/security/policy)
