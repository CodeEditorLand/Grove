//! WASM Memory Manager
//!
//! Manages memory allocation, deallocation, and limits for WebAssembly
//! instances. Enforces memory constraints and provides tracking for debugging.

use std::sync::{
	Arc,
	atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::dev_log;
#[allow(unused_imports)]
use wasmtime::{Memory, MemoryType};

/// Memory limits for WASM instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
	/// Maximum memory per instance in MB
	pub max_memory_mb:u64,
	/// Initial memory allocation per instance in MB
	pub initial_memory_mb:u64,
	/// Maximum table size (number of elements)
	pub max_table_size:u32,
	/// Maximum number of memory instances
	pub max_memories:usize,
	/// Maximum number of table instances
	pub max_tables:usize,
	/// Maximum number of instances that can be created
	pub max_instances:usize,
}

impl Default for MemoryLimits {
	fn default() -> Self {
		Self {
			max_memory_mb:512,
			initial_memory_mb:64,
			max_table_size:1024,
			max_memories:10,
			max_tables:10,
			max_instances:100,
		}
	}
}

impl MemoryLimits {
	/// Create custom memory limits
	pub fn new(max_memory_mb:u64, initial_memory_mb:u64, max_instances:usize) -> Self {
		Self { max_memory_mb, initial_memory_mb, max_instances, ..Default::default() }
	}

	/// Convert max memory to bytes
	pub fn max_memory_bytes(&self) -> u64 { self.max_memory_mb * 1024 * 1024 }

	/// Convert initial memory to bytes
	pub fn initial_memory_bytes(&self) -> u64 { self.initial_memory_mb * 1024 * 1024 }

	/// Validate memory request
	pub fn validate_request(&self, requested_bytes:u64, current_usage:u64) -> Result<()> {
		if current_usage + requested_bytes > self.max_memory_bytes() {
			return Err(anyhow::anyhow!(
				"Memory request exceeds limit: {} + {} > {} bytes",
				current_usage,
				requested_bytes,
				self.max_memory_bytes()
			));
		}
		Ok(())
	}
}

/// Memory allocation record for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
	/// Unique allocation identifier
	pub id:String,
	/// Instance ID that owns this memory
	pub instance_id:String,
	/// Memory type/identifier
	pub memory_type:String,
	/// Amount of memory allocated in bytes
	pub size_bytes:u64,
	/// Maximum size this allocation can grow to
	pub max_size_bytes:u64,
	/// Allocation timestamp
	pub allocated_at:u64,
	/// Whether this memory is shared
	pub is_shared:bool,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
	/// Total memory allocated in bytes
	pub total_allocated:u64,
	/// Total memory allocated in MB
	pub total_allocated_mb:f64,
	/// Number of memory allocations
	pub allocation_count:usize,
	/// Number of memory deallocations
	pub deallocation_count:usize,
	/// Peak memory usage in bytes
	pub peak_memory_bytes:u64,
	/// Peak memory usage in MB
	pub peak_memory_mb:f64,
}

impl Default for MemoryStats {
	fn default() -> Self {
		Self {
			total_allocated:0,
			total_allocated_mb:0.0,
			allocation_count:0,
			deallocation_count:0,
			peak_memory_bytes:0,
			peak_memory_mb:0.0,
		}
	}
}

impl MemoryStats {
	/// Update stats with new allocation
	pub fn record_allocation(&mut self, size_bytes:u64) {
		self.total_allocated += size_bytes;
		self.allocation_count += 1;
		if self.total_allocated > self.peak_memory_bytes {
			self.peak_memory_bytes = self.total_allocated;
		}
		self.total_allocated_mb = self.total_allocated as f64 / (1024.0 * 1024.0);
		self.peak_memory_mb = self.peak_memory_bytes as f64 / (1024.0 * 1024.0);
	}

	/// Update stats with deallocation
	pub fn record_deallocation(&mut self, size_bytes:u64) {
		self.total_allocated = self.total_allocated.saturating_sub(size_bytes);
		self.deallocation_count += 1;
		self.total_allocated_mb = self.total_allocated as f64 / (1024.0 * 1024.0);
	}
}

/// WASM Memory Manager
#[derive(Debug)]
pub struct MemoryManagerImpl {
	limits:MemoryLimits,
	allocations:Vec<MemoryAllocation>,
	stats:Arc<MemoryStats>,
	peak_usage:Arc<AtomicU64>,
}

impl MemoryManagerImpl {
	/// Create a new memory manager with the given limits
	pub fn new(limits:MemoryLimits) -> Self {
		Self {
			limits,
			allocations:Vec::new(),
			stats:Arc::new(MemoryStats::default()),
			peak_usage:Arc::new(AtomicU64::new(0)),
		}
	}

	/// Get the current memory limits
	pub fn limits(&self) -> &MemoryLimits { &self.limits }

	/// Get current memory statistics
	pub fn stats(&self) -> &MemoryStats { &self.stats }

	/// Get peak memory usage
	pub fn peak_usage_bytes(&self) -> u64 { self.peak_usage.load(Ordering::Relaxed) }

	/// Get peak memory usage in MB
	pub fn peak_usage_mb(&self) -> f64 { self.peak_usage.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0) }

	/// Get current memory usage in bytes
	pub fn current_usage_bytes(&self) -> u64 { self.allocations.iter().map(|a| a.size_bytes).sum() }

	/// Get current memory usage in MB
	pub fn current_usage_mb(&self) -> f64 { self.current_usage_bytes() as f64 / (1024.0 * 1024.0) }

	/// Check if memory can be allocated
	pub fn can_allocate(&self, requested_bytes:u64) -> bool {
		let current = self.current_usage_bytes();
		current + requested_bytes <= self.limits.max_memory_bytes()
	}

	/// Allocate memory for a WASM instance
	pub fn allocate_memory(&mut self, instance_id:&str, memory_type:&str, requested_bytes:u64) -> Result<u64> {
		dev_log!("wasm", "Allocating {} bytes for instance {} (type: {})", requested_bytes, instance_id, memory_type);

		let current_usage = self.current_usage_bytes();

		// Validate against limits
		self.limits
			.validate_request(requested_bytes, current_usage)
			.context("Memory allocation validation failed")?;

		// Check allocation count limit
		if self.allocations.len() >= self.limits.max_memories {
			return Err(anyhow::anyhow!(
				"Maximum number of memory allocations reached: {}",
				self.limits.max_memories
			));
		}

		// Create allocation record
		let allocation = MemoryAllocation {
			id:format!("alloc-{}", uuid::Uuid::new_v4()),
			instance_id:instance_id.to_string(),
			memory_type:memory_type.to_string(),
			size_bytes:requested_bytes,
			max_size_bytes:self.limits.max_memory_bytes() - current_usage,
			allocated_at:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
			is_shared:false,
		};

		self.allocations.push(allocation);

		// Update stats
		Arc::make_mut(&mut self.stats).record_allocation(requested_bytes);

		// Update peak usage
		let new_peak = self.current_usage_bytes();
		let current_peak = self.peak_usage.load(Ordering::Relaxed);
		if new_peak > current_peak {
			self.peak_usage.store(new_peak, Ordering::Relaxed);
		}

		dev_log!("wasm", "Memory allocated successfully. Total usage: {} MB", self.current_usage_mb());

		Ok(requested_bytes)
	}

	/// Deallocate memory for a WASM instance
	pub fn deallocate_memory(&mut self, instance_id:&str, memory_id:&str) -> Result<bool> {
		dev_log!("wasm", "Deallocating memory {} for instance {}", memory_id, instance_id);

		let pos = self
			.allocations
			.iter()
			.position(|a| a.instance_id == instance_id && a.id == memory_id);

		if let Some(pos) = pos {
			let allocation = self.allocations.remove(pos);

			// Update stats
			Arc::make_mut(&mut self.stats).record_deallocation(allocation.size_bytes);

			dev_log!("wasm", "Memory deallocated successfully. Remaining usage: {} MB", self.current_usage_mb());

			Ok(true)
		} else {
			dev_log!("wasm", "warn: memory allocation not found: {} for instance {}", memory_id, instance_id);
			Ok(false)
		}
	}

	/// Deallocate all memory for an instance
	pub fn deallocate_all_for_instance(&mut self, instance_id:&str) -> usize {
		dev_log!("wasm", "Deallocating all memory for instance {}", instance_id);

		let initial_count = self.allocations.len();

		self.allocations.retain(|a| a.instance_id != instance_id);

		let deallocated_count = initial_count - self.allocations.len();

		if deallocated_count > 0 {
			dev_log!("wasm", "Deallocated {} memory allocations for instance {}", deallocated_count, instance_id);
		}

		deallocated_count
	}

	/// Grow existing memory allocation
	pub fn grow_memory(&mut self, instance_id:&str, memory_id:&str, additional_bytes:u64) -> Result<u64> {
		dev_log!("wasm", "Growing memory {} for instance {} by {} bytes", memory_id, instance_id, additional_bytes);

		// Calculate current usage before mutable borrow
		let current_usage = self.current_usage_bytes();

		let allocation = self
			.allocations
			.iter_mut()
			.find(|a| a.instance_id == instance_id && a.id == memory_id)
			.ok_or_else(|| anyhow::anyhow!("Memory allocation not found"))?;

		// Validate against limits
		self.limits
			.validate_request(additional_bytes, current_usage)
			.context("Memory growth validation failed")?;

		allocation.size_bytes += additional_bytes;

		dev_log!("wasm", "Memory grown successfully. New size: {} bytes", allocation.size_bytes);

		Ok(allocation.size_bytes)
	}

	/// Get all allocations for an instance
	pub fn get_allocations_for_instance(&self, instance_id:&str) -> Vec<&MemoryAllocation> {
		self.allocations.iter().filter(|a| a.instance_id == instance_id).collect()
	}

	/// Check if memory limits are exceeded
	pub fn is_exceeded(&self) -> bool { self.current_usage_bytes() > self.limits.max_memory_bytes() }

	/// Get memory usage percentage
	pub fn usage_percentage(&self) -> f64 {
		(self.current_usage_bytes() as f64 / self.limits.max_memory_bytes() as f64) * 100.0
	}

	/// Reset all allocations and stats (use with caution)
	pub fn reset(&mut self) {
		self.allocations.clear();
		self.stats = Arc::new(MemoryStats::default());
		self.peak_usage.store(0, Ordering::Relaxed);
		dev_log!("wasm", "Memory manager reset");
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_memory_limits_default() {
		let limits = MemoryLimits::default();
		assert_eq!(limits.max_memory_mb, 512);
		assert_eq!(limits.initial_memory_mb, 64);
	}

	#[test]
	fn test_memory_limits_custom() {
		let limits = MemoryLimits::new(1024, 128, 50);
		assert_eq!(limits.max_memory_mb, 1024);
		assert_eq!(limits.initial_memory_mb, 128);
		assert_eq!(limits.max_instances, 50);
	}

	#[test]
	fn test_memory_limits_validation() {
		let limits = MemoryLimits::new(100, 10, 10);

		// Valid request
		assert!(limits.validate_request(50, 0).is_ok());

		// Exceeds limit
		assert!(limits.validate_request(150, 0).is_err());
		assert!(limits.validate_request(50, 60).is_err());
	}

	#[test]
	fn test_memory_manager_creation() {
		let limits = MemoryLimits::default();
		let manager = MemoryManagerImpl::new(limits);
		assert_eq!(manager.current_usage_bytes(), 0);
		assert_eq!(manager.allocations.len(), 0);
	}

	#[test]
	fn test_memory_allocation() {
		let limits = MemoryLimits::default();
		let mut manager = MemoryManagerImpl::new(limits);

		let result = manager.allocate_memory("test-instance", "heap", 1024);
		assert!(result.is_ok());
		assert_eq!(manager.current_usage_bytes(), 1024);
		assert_eq!(manager.allocations.len(), 1);
	}

	#[test]
	fn test_memory_deallocation() {
		let limits = MemoryLimits::default();
		let mut manager = MemoryManagerImpl::new(limits);

		manager.allocate_memory("test-instance", "heap", 1024).unwrap();
		let allocation = &manager.allocations[0];
		let memory_id = allocation.id.clone();

		let result = manager.deallocate_memory("test-instance", &memory_id);
		assert!(result.is_ok());
		assert_eq!(manager.current_usage_bytes(), 0);
		assert_eq!(manager.allocations.len(), 0);
	}

	#[test]
	fn test_memory_stats() {
		let mut stats = MemoryStats::default();
		stats.record_allocation(1024);
		assert_eq!(stats.allocation_count, 1);
		assert_eq!(stats.total_allocated, 1024);

		stats.record_deallocation(512);
		assert_eq!(stats.deallocation_count, 1);
		assert_eq!(stats.total_allocated, 512);
	}

	#[test]
	fn test_memory_usage_percentage() {
		let limits = MemoryLimits::new(1000, 0, 0);
		let mut manager = MemoryManagerImpl::new(limits);

		manager.allocate_memory("test", "heap", 500).unwrap();
		assert_eq!(manager.usage_percentage(), 50.0);
	}
}
