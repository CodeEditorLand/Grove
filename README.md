# Grove

[![Crates.io](https://img.shields.io/crates/v/grove)](https://crates.io/crates/grove)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Grove** is a high-performance Rust/WebAssembly extension host designed to complement Cocoon (Node.js) for running native Rust and WASM VS Code extensions.

## Overview

Grove provides a secure, sandboxed environment for running VS Code extensions compiled to WebAssembly or native Rust, offering:

- **WASM Support**: Full WebAssembly runtime with WASMtime
- **Standalone Operation**: Can run independently or connect to Mountain via gRPC
- **Cross-Platform**: Native support for macOS, Linux, and Windows
- **Cocoon Compatible**: Shares API surface and semantics with Node.js Cocoon host
- **Multiple Transport**: gRPC, IPC, and direct WASM transport strategies
- **Secure Sandboxing**: WASMtime-based isolation for untrusted extensions

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Grove                               │
├─────────────────────────────────────────────────────────────┤
│  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │ Extension  │  │  Activation│  │   API Bridge         │  │
│  │   Host     │─▶│  Manager   │─▶│  (VS Code API)       │  │
│  └────────────┘  └────────────┘  └──────────────────────┘  │
│         │                                              ▲     │
│         ▼                                              │     │
│  ┌─────────────────────────────────────────────────────┤     │
│  │     WASM Runtime (WASMtime)                         │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │     │
│  │  │Module    │  │ Memory   │  │   Host Bridge    │  │     │
│  │  │ Loader   │  │ Manager  │  │   (Communication)│  │     │
│  │  └──────────┘  └──────────┘  └──────────────────┘  │     │
│  └─────────────────────────────────────────────────────┘     │
│         │                                                      │
│         ▼                                                      │
│  ┌──────────────────────────────────────────────────────┐     │
│  │  Transport Layer                                      │     │
│  │  ┌─────────┐  ┌─────────┐  ┌────────────────────┐   │     │
│  │  │ gRPC    │  │   IPC   │  │  Direct WASM       │◀──┘     │
│  │  └─────────┘  └─────────┘  └────────────────────┘        │
│  └──────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Compatibility

Grove is designed to be compatible with:

- **Cocoon**: Shares VS Code API surface, activation semantics, and manifest parsing
- **VS Code**: Implements vscode.d.ts type definitions
- **Mountain**: Integrates via GroveService gRPC protocol (Vine.proto)

## Building

### Prerequisites

- Rust 1.75 or later
- Protocol Buffer compiler (optional, for proto file modifications)

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

## Usage

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

## Project Structure

```
Element/Grove/
├── Source/
│   ├── lib.rs                    # Library root
│   ├── main.rs                   # Binary entry point
│   ├── Binary/                   # Binary initialization
│   ├── Host/                     # Extension host core
│   ├── WASM/                     # WebAssembly runtime
│   ├── API/                      # VS Code API facade
│   ├── Transport/                # Communication layer
│   ├── Protocol/                 # Protocol handling
│   ├── Services/                 # Host services
│   └── Common/                   # Shared utilities
├── Proto/
│   └── Grove.proto              # Grove-specific protocol
└── Tests/                       # Integration tests
```

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

## Features

- `default`: Enables grpc and wasm features
- `grpc`: gRPC transport support
- `wasm`: WebAssembly runtime support
- `ipc`: Inter-process communication (Unix only)
- `all`: All features enabled

## Security

Grove provides security through:

1. **WASM Sandboxing**: Isolated execution environment via WASMtime
2. **Memory Limits**: Configurable memory constraints for extensions
3. **Resource Controls**: CPU and resource throttling
4. **Type Safety**: Rust's ownership system ensures memory safety
5. **Secure API**: Controlled access to host functions

## Performance

- Zero-cost abstractions via Rust
- LTO (Link Time Optimization) in release builds
- Efficient WASM compilation and instantiation
- Asynchronous I/O via Tokio

## License

MIT License - see [LICENSE](https://github.com/CodeEditorLand/Land/tree/Current/Element/LICENSE) file for details

## Contributing

Please see [CONTRIBUTING.md](https://github.com/CodeEditorLand/Land/tree/Current/CONTRIBUTING.md) for guidelines.

## Related Projects

- [Cocoon](https://github.com/CodeEditorLand/Cocoon/tree/Current/) - Node.js Extension Host
- [Mountain](https://github.com/CodeEditorLand/Land/tree/Current/Documentation/Architecture/components/Mountain.md) - Core VS Code implementation
- [Vine](https://github.com/CodeEditorLand/Land/tree/Current/Documentation/Architecture/components/Vine.md) - Communication protocol
