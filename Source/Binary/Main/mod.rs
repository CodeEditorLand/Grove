//! Main Module (Binary)
//!
//! Entry point for the standalone Grove binary.
//! Provides CLI argument parsing and initialization.

pub mod Entry;

/// Main entry result
pub type MainResult<T> = anyhow::Result<T>;

/// CLI arguments wrapper
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Mode of operation
    pub mode: String,
    /// Extension path
    pub extension: Option<String>,
    /// Transport type
    pub transport: String,
    /// gRPC address
    pub grpc_address: String,
    /// Mountain address (for service mode)
    pub mountain_address: String,
    /// Enable WASI
    pub wasi: bool,
    /// Memory limit in MB
    pub memory_limit_mb: u64,
    /// Max execution time in ms
    pub max_execution_time_ms: u64,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            mode: "standalone".to_string(),
            extension: None,
            transport: "wasm".to_string(),
            grpc_address: "127.0.0.1:50051".to_string(),
            mountain_address: "127.0.0.1:50050".to_string(),
            wasi: true,
            memory_limit_mb: 512,
            max_execution_time_ms: 30000,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_default() {
        let args = CliArgs::default();
        assert_eq!(args.mode, "standalone");
        assert_eq!(args.transport, "wasm");
        assert!(args.wasi);
        assert_eq!(args.memory_limit_mb, 512);
    }
}
