<table>
<tr>
<td align="left" valign="middle">
<h3 align="left"> Grove</h3>
</td>
<td align="left" valign="middle">
<h3 align="left">
🌳
</h3>
</td>
<td align="left" valign="middle">
<h3 align="left"> + </h3>
</td>
<td align="left" valign="middle">
<h3 align="left">
<a href="https://Editor.Land" target="_blank">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://PlayForm.Cloud/Dark/Image/GitHub/Land.svg">
<source media="(prefers-color-scheme: light)" srcset="https://PlayForm.Cloud/Image/GitHub/Land.svg">
<img width="28" alt="Land Logo" src="https://PlayForm.Cloud/Image/GitHub/Land.svg">
</picture>
</a>
</h3>
</td>
<td align="left" valign="middle">
<h3 align="left">
<a href="https://Editor.Land" target="_blank">
Land
</a>
</h3>
</td>
<td align="left" valign="middle">
<h3 align="left">
🏞️
</h3>
</td>
</tr>
</table>

---

# **Grove** 🌳

The Native Rust/WASM Extension Host for Land 🏞️

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Grove/tree/Current/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/Grove.svg)](https://crates.io/crates/Grove)
[![Rust Version](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![WASMtime Version](https://img.shields.io/badge/WASMtime-v20-blue.svg)](https://wasmtime.dev/)

Welcome to **Grove**, the high-performance Rust/WebAssembly extension host for
the **Land Code Editor**. Grove complements `Cocoon` (Node.js) by providing a
native environment for running Rust and WASM-compiled VS Code extensions. It
offers secure sandboxing through WASMtime, multiple transport strategies (gRPC,
IPC, WASM), and full compatibility with the VS Code API surface.

**Grove** is engineered to:

1. **Provide Native Extension Hosting:** Execute Rust extensions with zero
   overhead through static linking or WASM sandboxing.
2. **Enable Secure Sandboxing:** Isolate untrusted extensions using WASMtime's
   capability-based security model.
3. **Support Multiple Transports:** Communicate with `Mountain` via gRPC, IPC,
   or direct WASM host functions.
4. **Maintain Cocoon Compatibility:** Share the same VS Code API surface and
   activation semantics for seamless extension porting.

---

## Key Features 🔐

- **WASM Runtime Integration:** Full WebAssembly support through WASMtime,
  enabling secure sandboxing of untrusted extensions with capability-based
  security.
- **Multiple Transport Strategies:** Support for gRPC, IPC, and direct WASM host
  function communication with the `Mountain` backend.
- **Standalone Operation:** Can run independently as a standalone process or
  connect to `Mountain` via gRPC for distributed deployment.
- **Cross-Platform Support:** Native support for macOS, Linux, and Windows with
  platform-specific optimizations.
- **VS Code API Compatibility:** Implements vscode.d.ts type definitions for
  seamless extension porting from the Node.js ecosystem.
- **Secure Sandboxing:** WASMtime-based isolation with configurable memory
  limits and resource controls for untrusted code.

---

## Core Architecture Principles 🏗️

| Principle                 | Description                                                                                            | Key Components Involved                                                   |
| :------------------------ | :----------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------ |
| **Security First**        | Isolate extensions using WASMtime's capability-based security model with configurable resource limits. | `WASM/Runtime`, `WASM/MemoryManager`                                      |
| **Transport Agnosticism** | Support multiple communication strategies (gRPC, IPC, WASM) for flexible deployment scenarios.         | `Transport/Strategy`, `Transport/gRPCTransport`, `Transport/IPCTransport` |
| **API Compatibility**     | Maintain high fidelity with the VS Code API surface to enable seamless extension porting.              | `API/vscode`, `API/types`, `APIBridge`                                    |
| **Performance**           | Zero-cost abstractions via Rust with LTO optimization for maximum execution speed.                     | `Host/ExtensionHost`, `WASM/ModuleLoader`                                 |
| **Composability**         | Modular architecture with clear separation between host, transport, and API layers.                    | `Host/*`, `Transport/*`, `API/*`                                          |

---

## `Grove` in the Land Ecosystem 🌳 + 🏞️

This diagram illustrates `Grove`'s role as the native Rust/WASM extension host
alongside `Cocoon` (Node.js).

| Component                     | Role & Key Responsibilities                                                   |
| :---------------------------- | :---------------------------------------------------------------------------- |
| **Extension Host Controller** | Manages extension discovery, loading, and lifecycle for Rust/WASM extensions. |
| **WASM Runtime**              | Provides secure sandboxing through WASMtime with memory and resource limits.  |
| **Transport Layer**           | Handles communication with `Mountain` via gRPC, IPC, or direct WASM calls.    |
| **API Bridge**                | Implements the VS Code API facade for extension compatibility.                |
| **Activation Manager**        | Manages extension activation events and contribution points.                  |

---

## Overview 📖

Grove provides a secure, sandboxed environment for running VS Code extensions
compiled to WebAssembly or native Rust, offering:

- **WASM Support**: Full WebAssembly runtime with WASMtime
- **Standalone Operation**: Can run independently or connect to Mountain via
  gRPC
- **Cross-Platform**: Native support for macOS, Linux, and Windows
- **Cocoon Compatible**: Shares API surface and semantics with Node.js Cocoon
  host
- **Multiple Transport**: gRPC, IPC, and direct WASM transport strategies
- **Secure Sandboxing**: WASMtime-based isolation for untrusted extensions

---

## System Architecture Diagram 🏗️

This diagram illustrates `Grove`'s internal architecture and its place within
the broader Land ecosystem.

```mermaid
graph LR
classDef grove fill:#ccf,stroke:#333,stroke-width:2px;
classDef mountain fill:#f9f,stroke:#333,stroke-width:2px;
classDef wasm fill:#cfc,stroke:#333,stroke-width:1px;
classDef transport fill:#ff9,stroke:#333,stroke-width:1px,stroke-dasharray: 5 5;

subgraph "Grove 🌳 (Rust/WASM Extension Host)"
direction TB
ExtensionHost["Extension Host Controller"]:::grove
ActivationMgr["Activation Manager"]:::grove
APIBridge["VS Code API Bridge"]:::grove
WASMRuntime["WASM Runtime (WASMtime)"]:::wasm
TransportLayer["Transport Layer"]:::transport

ExtensionHost --> ActivationMgr
ActivationMgr --> APIBridge
APIBridge --> WASMRuntime
WASMRuntime --> TransportLayer
end

subgraph "Mountain ⛰️ (Rust/Tauri Backend)"
VineGRPC["Vine gRPC Server"]:::mountain
end

TransportLayer -- gRPC/IPC --> VineGRPC
```

---

## Compatibility ✅

Grove is designed to be compatible with:

- **Cocoon**: Shares VS Code API surface, activation semantics, and manifest
  parsing
- **VS Code**: Implements vscode.d.ts type definitions
- **Mountain**: Integrates via GroveService gRPC protocol (Vine.proto)

---

## Building

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

---

## Usage 🚀

### Standalone Mode

```bash
# Run Grove as a standalone host
cargo run --bin grove -- --standalone

# Run with specific extension
cargo run --bin grove -- --extension /path/to/extension
```

### As Library

```rust
use grove::ExtensionHost;
use grove::Transport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = ExtensionHost::new(Transport::default()).await?;
    host.load_extension("/path/to/extension").await?;
    host.activate().await?;
    Ok(())
}
```

---

## Deep Dive & Component Breakdown 🔬

To understand how `Grove`'s internal components interact to provide the
high-fidelity WASM and Rust extension hosting environment, see the following
source files:

- **[`Source/Host/`](https://github.com/CodeEditorLand/Grove/tree/Current/Source/Host/)** -
  Core extension host controller and lifecycle management
- **[`Source/WASM/`](https://github.com/CodeEditorLand/Grove/tree/Current/Source/WASM/)** -
  WebAssembly runtime integration with WASMtime
- **[`Source/API/`](https://github.com/CodeEditorLand/Grove/tree/Current/Source/API/)** -
  VS Code API facade and type definitions
- **[`Source/Transport/`](https://github.com/CodeEditorLand/Grove/tree/Current/Source/Transport/)** -
  Communication strategies (gRPC, IPC, WASM)
- **[`Proto/Grove.proto`](Proto/Grove.proto)** - gRPC protocol definitions for
  Mountain integration

The source files explain the role of each module, the WASM sandboxing approach,
and the communication patterns with the Mountain backend.

---

## Project Structure 🗺️

```
Element/Grove/
├── Source/
│   ├── lib.rs # Library root
│   ├── main.rs # Binary entry point
│   ├── Binary/ # Binary initialization
│   ├── Host/ # Extension host core
│   ├── WASM/ # WebAssembly runtime
│   ├── API/ # VS Code API facade
│   ├── Transport/ # Communication layer
│   ├── Protocol/ # Protocol handling
│   ├── Services/ # Host services
│   └── Common/ # Shared utilities
├── Proto/
│   └── Grove.proto # Grove-specific protocol
└── Documentation/Rust/doc/ # Generated Rust documentation
```

---

## Modules

### Host

Core extension hosting functionality:

- `ExtensionHost`: Main host controller
- `ExtensionManager`: Extension discovery and loading
- `Activation`: Extension activation events
- `Lifecycle`: Extension lifecycle management
- `APIBridge`: VS Code API implementation

### WASM

WebAssembly runtime integration:

- `Runtime`: WASMtime engine and store management
- `ModuleLoader`: WASM module compilation and instantiation
- `MemoryManager`: WASM memory allocation and management
- `HostBridge`: Host-WASM function communication
- `FunctionExport`: Export host functions to WASM

### Transport

Communication strategies:

- `Strategy`: Transport strategy trait
- `gRPCTransport`: gRPC-based communication with Mountain
- `IPCTransport`: Inter-process communication
- `WASMTransport`: Direct WASM communication

### API

VS Code API implementation:

- `vscode`: VS Code API facade
- `types`: Common API type definitions

### Protocol

Protocol handling:

- `SpineConnection`: Spine protocol client connection

---

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With output
cargo test -- --nocapture
```

### Code Style

This project uses standard Rust formatting:

```bash
cargo fmt
cargo clippy
```

---

## Features

- `default`: Enables grpc and wasm features
- `grpc`: gRPC transport support
- `wasm`: WebAssembly runtime support
- `ipc`: Inter-process communication (Unix only)
- `all`: All features enabled

---

## Security 🔒

Grove provides security through:

1. **WASM Sandboxing:** Isolated execution environment via WASMtime.
2. **Memory Limits:** Configurable memory constraints for extensions.
3. **Resource Controls:** CPU and resource throttling.
4. **Type Safety:** Rust's ownership system ensures memory safety.
5. **Secure API:** Controlled access to host functions.

---

## Performance ⚡

- Zero-cost abstractions via Rust
- LTO (Link Time Optimization) in release builds
- Efficient WASM compilation and instantiation
- Asynchronous I/O via Tokio

---

## License ⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license.

You are free to use, modify, distribute, and build upon this work for any
purpose, without any restrictions. For the full legal text, see the
[`LICENSE`](https://github.com/CodeEditorLand/Grove/tree/Current/) file.

---

## Changelog 📜

Stay updated with our progress! See
[`CHANGELOG.md`](https://github.com/CodeEditorLand/Grove/tree/Current/) for a
history of changes specific to **Grove**.

---

## Funding & Acknowledgements 🙏🏻

**Grove** is a core element of the **Land** ecosystem. This project is funded
through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
[Next Generation Internet](https://ngi.eu) program. Learn more at the
[NLnet project page](https://NLnet.NL/project/Land).

The project is operated by PlayForm, based in Sofia, Bulgaria.

PlayForm acts as the open-source steward for Code Editor Land under the NGI0
Commons Fund grant.

<table>
	<thead>
		<tr>
			<th align="left"><strong>Land</strong></th>
			<th align="left"><strong>PlayForm</strong></th>
			<th align="left"><strong>NLnet</strong></th>
			<th align="left"><strong>NGI0 Commons Fund</strong></th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td align="left" valign="middle">
				<a href="https://Editor.Land">
					<img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://PlayForm.Cloud">
					<img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL">
					<img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL/commonsfund">
					<img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund">
				</a>
			</td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@Editor.Land](mailto:Source/Open@Editor.Land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Grove) |
[Report an Issue](https://github.com/CodeEditorLand/Grove/issues) |
[Security Policy](https://github.com/CodeEditorLand/Grove/security/policy)
