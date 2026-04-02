# Grove — Deep Dive

This document provides the technical foundation for the Grove Rust/WASM
extension host within the Land ecosystem. **Grove** provides a native,
sandboxed environment for running VS Code extensions compiled to WebAssembly or
native Rust, complementing the Node.js-based Cocoon host.

---

## Architecture

Grove is organized into five layers: the extension host controller that manages
the lifecycle, the WASM runtime that provides sandboxed execution, the VS Code
API bridge that presents the standard extension API, the transport layer that
communicates with Mountain, and shared utility modules.

```mermaid
graph TB
    subgraph "Grove — Rust/WASM Extension Host"
        Main["main.rs / lib.rs\nBinary + Library entry"]
        Binary["Binary/\nCLI and startup"]
        Host["Host/\nExtension Host Controller"]
        ExtHost["Host/ExtensionHost.rs\nMain controller"]
        ExtMgr["Host/ExtensionManager.rs\nDiscovery and loading"]
        Activation["Host/Activation.rs\nActivation events"]
        Lifecycle["Host/Lifecycle.rs\nExtension lifecycle"]
        APIBridge["Host/APIBridge.rs\nVS Code API facade"]
        WASM["WASM/\nWebAssembly Runtime"]
        Runtime["WASM/Runtime.rs\nWASMtime engine + store"]
        ModuleLoader["WASM/ModuleLoader.rs\nModule compilation"]
        MemoryMgr["WASM/MemoryManager.rs\nMemory allocation"]
        HostBridge["WASM/HostBridge.rs\nHost-WASM function bridge"]
        Transport["Transport/\nCommunication Strategies"]
        GRPCTransport["Transport/gRPCTransport.rs\ngRPC via Mountain"]
        IPCTransport["Transport/IPCTransport.rs\nLocal IPC"]
        WASMTransport["Transport/WASMTransport.rs\nDirect WASM calls"]
        API["API/\nvscode API + types"]
        Protocol["Protocol/\nSpine connection"]
        Services["Services/\nHost services"]
        Common["Common/\nShared utilities"]
    end

    subgraph "Mountain — Rust Backend"
        VineGRPC["Vine gRPC Server"]
    end

    Main --> Binary
    Binary --> Host
    Host --> ExtHost
    ExtHost --> ExtMgr
    ExtHost --> Activation
    ExtHost --> Lifecycle
    ExtHost --> APIBridge
    APIBridge --> API
    APIBridge --> WASM
    WASM --> Runtime
    WASM --> ModuleLoader
    WASM --> MemoryMgr
    WASM --> HostBridge
    ExtHost --> Transport
    Transport --> GRPCTransport
    Transport --> IPCTransport
    Transport --> WASMTransport
    ExtHost --> Protocol
    GRPCTransport --> VineGRPC
```

---

## Key Modules

| Path | Description |
| :--- | :--- |
| `Source/lib.rs` | Library root; exports public API for integration and testing |
| `Source/main.rs` | Binary entry point; parses CLI and starts the extension host |
| `Source/Binary/` | CLI argument handling and standalone vs connected mode selection |
| `Source/Host/ExtensionHost.rs` | Main controller: coordinates extension loading, activation, and service provision |
| `Source/Host/ExtensionManager.rs` | Extension discovery, manifest parsing, and loading |
| `Source/Host/Activation.rs` | Activation event processing (`onLanguage`, `onCommand`, etc.) |
| `Source/Host/Lifecycle.rs` | Extension activate/deactivate lifecycle management |
| `Source/Host/APIBridge.rs` | VS Code API facade implementation for Rust/WASM extensions |
| `Source/WASM/Runtime.rs` | WASMtime engine and store management with memory limits |
| `Source/WASM/ModuleLoader.rs` | WASM module compilation, caching, and instantiation |
| `Source/WASM/MemoryManager.rs` | WASM linear memory allocation and bounds enforcement |
| `Source/WASM/HostBridge.rs` | Host function exports to WASM modules |
| `Source/WASM/FunctionExport.rs` | Registered host functions available to extension WASM code |
| `Source/Transport/Strategy.rs` | Transport strategy trait for pluggable communication |
| `Source/Transport/gRPCTransport.rs` | gRPC communication with Mountain via `tonic` |
| `Source/Transport/IPCTransport.rs` | Unix/Windows IPC transport for same-host communication |
| `Source/Transport/WASMTransport.rs` | Direct in-process WASM function call transport |
| `Source/API/vscode.rs` | VS Code API surface implementation |
| `Source/API/types.rs` | Common API type definitions matching VS Code's TypeScript types |
| `Source/Protocol/SpineConnection.rs` | Spine protocol client for extension host coordination |
| `Proto/Grove.proto` | Grove-specific gRPC protocol definitions |

---

## Data Flow

```mermaid
sequenceDiagram
    participant Mountain as Mountain Core
    participant Grove as Grove Extension Host
    participant WASM as WASM Runtime
    participant Extension as WASM Extension

    Mountain->>Grove: GroveService.ActivateExtension (Grove.proto)
    Grove->>Grove: ExtensionManager.load(manifest)
    Grove->>WASM: ModuleLoader.compile(wasm_bytes)
    WASM->>Grove: Compiled module
    Grove->>WASM: Runtime.instantiate(module, host_functions)
    WASM->>Extension: WASM module initialized
    Extension->>WASM: Call host function (e.g. vscode.workspace.readFile)
    WASM->>Grove: HostBridge dispatch
    Grove->>Mountain: Forward service call via Transport
    Mountain->>Grove: Service result
    Grove->>WASM: Return value to extension
    WASM->>Extension: Result available
```

---

## Integration Points

| Connecting Element | Direction | Mechanism | Description |
| :--- | :--- | :--- | :--- |
| **Mountain** | Bidirectional | gRPC via Grove.proto | Mountain activates extensions; Grove forwards API calls back to Mountain |
| **Vine** | Inbound | Protocol definition | Grove.proto extends Vine's service definitions for Grove-specific operations |
| **Cocoon** | Sibling | Shared API surface | Grove implements the same VS Code API surface as Cocoon for extension portability |

---

## Configuration

| Option | CLI Flag / Feature | Default | Description |
| :--- | :--- | :--- | :--- |
| Standalone mode | `--standalone` | off | Run without Mountain connection for testing |
| Extension path | `--extension` | unset | Load a specific extension directly |
| Transport | feature flag | gRPC | Select `grpc`, `ipc`, or `wasm` transport via Cargo features |
| Memory limit | runtime config | platform default | Per-extension WASM memory ceiling enforced by WASMtime |
| WASM build target | `wasm32-wasi` | native | Extensions must target `wasm32-wasi` for WASM mode |

**Cargo features:**

| Feature | Description |
| :--- | :--- |
| `grpc` | gRPC transport support (included in `default`) |
| `wasm` | WASMtime WASM runtime (included in `default`) |
| `ipc` | Unix/Windows IPC transport (Unix only) |
| `all` | All features enabled |
