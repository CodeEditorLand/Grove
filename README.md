# **Grove** 🌳

The Native Rust/WASM Extension Host for Land 🏞️

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Grove/blob/Current/LICENSE)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Crates.io](https://img.shields.io/crates/v/Grove.svg)](https://crates.io/crates/Grove)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Rust Version](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[<img src="https://editor.land/Image/WebAssembly.svg" width="14" alt="WebAssembly" />](https://webassembly.org/) [![WASMtime Version](https://img.shields.io/badge/WASMtime-v20-blue.svg)](https://wasmtime.dev/)

**[Rust API Documentation](https://Rust.Documentation.editor.land/Grove/)**

---

## Overview

Grove is a high-performance Rust/WebAssembly extension host for the Land Code Editor. It complements Cocoon (Node.js) by providing a native environment for running Rust and WASM-compiled VS Code extensions. Grove offers secure sandboxing through WASMtime, multiple transport strategies (gRPC, IPC, WASM), and full compatibility with the VS Code API surface. VS Code extensions run with full Node.js capabilities in a shared process — a malicious or buggy extension can access any file, make any network request, and read another extension's state. Grove solves this by enforcing sandboxing at the hardware level: an extension can only touch what you explicitly grant.

**Grove is engineered to:**

1. **Provide Native Extension Hosting:** Execute Rust extensions with zero overhead through static linking or WASM sandboxing.
2. **Enable Secure Sandboxing:** Isolate untrusted extensions using WASMtime's capability-based security model.
3. **Support Multiple Transports:** Communicate with Mountain via gRPC, IPC, or direct WASM host functions.
4. **Maintain Cocoon Compatibility:** Share the same VS Code API surface and activation semantics for seamless extension porting.

## Architecture

```mermaid
graph LR
    classDef grove    fill:#d0d8ff,stroke:#4a6fa5,stroke-width:2px,color:#001050;
    classDef mountain fill:#f0d0ff,stroke:#9b59b6,stroke-width:2px,color:#2c0050;
    classDef wasm     fill:#d4f5d4,stroke:#27ae60,stroke-width:2px,color:#0a3a0a;
    classDef transport fill:#fff3c0,stroke:#f39c12,stroke-width:1px,stroke-dasharray:5 5,color:#5a3e00;
    classDef cocoon   fill:#cce8ff,stroke:#2980b9,stroke-width:1px,stroke-dasharray:5 5,color:#003050;

    subgraph GROVE["Grove 🌳 - Rust/WASM Extension Host"]
        direction TB
        subgraph HOST["Host/ - Extension Lifecycle"]
            ExtHost["ExtensionHost.rs\n(main controller)"]:::grove
            ExtMgr["ExtensionManager.rs\n(discovery + loading)"]:::grove
            Activation["Activation.rs\n(activation events)"]:::grove
            Lifecycle["Lifecycle.rs"]:::grove
            APIBridge["APIBridge.rs\n(vscode.d.ts facade)"]:::grove
            ExtHost --> ExtMgr --> Activation --> Lifecycle
            Activation --> APIBridge
        end
        subgraph WASM_RT["WASM/ - WASMtime Runtime"]
            WASMRuntime["Runtime/ - WASMtime engine\n+ store management"]:::wasm
            ModLoader["ModuleLoader/ - WASM compile\n+ instantiation"]:::wasm
            MemMgr["MemoryManager/ - allocation\n+ configurable limits"]:::wasm
            HostBridge["HostBridge/ - host↔WASM\nfunction calls"]:::wasm
            WASMRuntime --> ModLoader
            ModLoader --> MemMgr
            WASMRuntime --> HostBridge
        end
        subgraph TRANSPORT["Transport/ - Strategy Pattern"]
            Strategy["Strategy.rs - trait"]:::transport
            gRPC["gRPCTransport.rs"]:::transport
            IPC["IPCTransport.rs"]:::transport
            WASMTrans["WASMTransport.rs"]:::transport
            Strategy --- gRPC
            Strategy --- IPC
            Strategy --- WASMTrans
        end
        subgraph PROTO["Protocol/"]
            SpineConn["SpineConnection.rs\n(Spine protocol client)"]:::grove
        end

        APIBridge --> WASMRuntime
        HostBridge --> Strategy
        SpineConn --> gRPC
    end

    subgraph MOUNTAIN["Mountain ⛰️"]
        VineGRPC["Vine gRPC Server 🌿"]:::mountain
    end

    subgraph COCOON["Cocoon 🦋 (complementary host)"]
        CocoonRef["Node.js extension host\n(same vscode API surface)"]:::cocoon
    end

    gRPC -- gRPC :50052 --> VineGRPC
    IPC -- Unix socket --> VineGRPC
    Grove -.shares API surface.-> CocoonRef
```

## Key Components

| Component | Path | Description |
| --------- | ---- | ----------- |
| ExtensionHost | `Source/Host/ExtensionHost.rs` | Main extension host controller — manages extension lifecycle |
| ExtensionManager | `Source/Host/ExtensionManager.rs` | Extension discovery and loading |
| Activation | `Source/Host/Activation.rs` | Extension activation events and contribution points |
| APIBridge | `Source/Host/APIBridge.rs` | VS Code API facade (vscode.d.ts compatibility) |
| WASM Runtime | `Source/WASM/Runtime/` | WASMtime engine and store management |
| ModuleLoader | `Source/WASM/ModuleLoader/` | WASM module compilation and instantiation |
| MemoryManager | `Source/WASM/MemoryManager/` | WASM memory allocation and configurable limits |
| HostBridge | `Source/WASM/HostBridge/` | Host-to-WASM function communication |
| FunctionExport | `Source/WASM/FunctionExport/` | Export host functions to WASM |
| Transport Strategy | `Source/Transport/Strategy.rs` | Transport strategy trait |
| gRPC Transport | `Source/Transport/gRPCTransport.rs` | gRPC-based communication with Mountain |
| IPC Transport | `Source/Transport/IPCTransport.rs` | Inter-process communication (Unix only) |
| WASM Transport | `Source/Transport/WASMTransport.rs` | Direct WASM communication |
| Spine Connection | `Source/Protocol/SpineConnection.rs` | Spine protocol client connection |

## In the Land Project

Grove communicates with Mountain via gRPC (port 50052), IPC (Unix socket), or direct WASM host function calls. It shares the same VS Code API surface as Cocoon, enabling seamless porting of extensions between the Node.js and WASM/Native hosting environments. Grove's Transport layer abstracts communication strategies, allowing flexible deployment — standalone process or integrated with Mountain's Vine gRPC server.

- **Architecture Principles:** Security First (WASMtime capability-based isolation), Transport Agnosticism (gRPC, IPC, WASM), Performance (zero-cost Rust abstractions with LTO), Composability (modular Host/Transport separation).

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Protocol Buffer compiler (optional, for proto file modifications)
- For WASM builds: `rustup target add wasm32-wasi`

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

- `default`: Enables `grpc` and `wasm` features
- `grpc`: gRPC transport support
- `wasm`: WebAssembly runtime support
- `ipc`: Inter-process communication (Unix only)
- `all`: All features enabled

### As Library

```rust
use grove::ExtensionHost;
use grove::Transport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Host = ExtensionHost::new(Transport::default()).await?;
    Host.load_extension("/path/to/extension").await?;
    Host.activate().await?;
    Ok(())
}
```

### Security

Grove provides security through WASM sandboxing (isolated execution via WASMtime), memory limits (configurable constraints for extensions), resource controls (CPU and resource throttling), type safety (Rust's ownership system), and secure API (controlled access to host functions via explicit capability grants).

### Compatibility

Grove is designed to be compatible with Cocoon (shares VS Code API surface, activation semantics, and manifest parsing), VS Code (implements vscode.d.ts type definitions), and Mountain (integrates via GroveService gRPC protocol using Vine.proto).

### Project Structure

```
Element/Grove/
├── Source/
│   ├── lib.rs           # Library root
│   ├── main.rs          # Binary entry point
│   ├── Binary/          # Binary initialization
│   ├── Host/            # Extension host core
│   │   ├── ExtensionHost    # Main host controller
│   │   ├── ExtensionManager # Extension discovery and loading
│   │   ├── Activation       # Extension activation events
│   │   ├── Lifecycle        # Extension lifecycle management
│   │   └── APIBridge        # VS Code API implementation
│   ├── WASM/            # WebAssembly runtime
│   │   ├── Runtime          # WASMtime engine and store management
│   │   ├── ModuleLoader     # WASM module compilation and instantiation
│   │   ├── MemoryManager    # WASM memory allocation and management
│   │   ├── HostBridge       # Host-WASM function communication
│   │   └── FunctionExport   # Export host functions to WASM
│   ├── Transport/       # Communication strategies
│   │   ├── Strategy         # Transport strategy trait
│   │   ├── gRPCTransport    # gRPC-based communication with Mountain
│   │   ├── IPCTransport     # Inter-process communication
│   │   └── WASMTransport    # Direct WASM communication
│   ├── Protocol/        # Protocol handling
│   │   └── SpineConnection  # Spine protocol client connection
└── Documentation/Rust/doc/
```

## API Reference

- [Rust API Documentation](https://Rust.Documentation.editor.land/Grove/)

## Related Documentation

- [Architecture Overview](https://Editor.Land/Doc/architecture)
- [Why WebAssembly](https://Editor.Land/Doc/why-webassembly)
- [Mountain](https://github.com/CodeEditorLand/Mountain) — Native desktop shell
- [Cocoon](https://github.com/CodeEditorLand/Cocoon) — Node.js extension host

---

## Funding

This project is funded through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by [NLnet](https://NLnet.NL) with financial support from the European Commission's Next Generation Internet program, under grant agreement No 101135429.

The project is operated by PlayForm, based in Sofia, Bulgaria. PlayForm acts as the open-source steward for Code Editor Land under the NGI0 Commons Fund grant.

| | |
| --- | --- |
| [![Land](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Dual/Land.svg)](https://Editor.Land) | [![PlayForm](https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg)](https://PlayForm.Cloud) |
| [![NLnet](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/NLnet.svg)](https://NLnet.NL) | [![NGI0](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/NGI0.svg)](https://NLnet.NL/commonsfund) |
