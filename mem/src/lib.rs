#![feature(ptr_internals, core_intrinsics)]
// Re-export utility modules for virtual memory allocations,
// pointer operations (alignment, ...) and an intrusive linked list
pub mod virtual_mem;
pub mod pointer_util;
pub mod freelist;

// Re-export modules that are requires and used as the basis for 
// the memory realm
pub mod allocators;
pub mod bounds_checker;
pub mod memory_realm;
