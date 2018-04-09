#![feature(ptr_internals, core_intrinsics)]

extern crate spark_core;

// Re-export utility modules for virtual memory allocations,
pub mod virtual_mem;

// Re-export modules that are requires and used as the basis for 
// the memory realm
pub mod allocators;
pub mod bounds_checker;
pub mod memory_realm;
