pub mod mem;
pub mod containers;

pub type BenchmarkFunction = fn();

pub static SCENARIOS: &'static [BenchmarkFunction] = &[
    // Allocators
    mem::linear_allocator_benchmarks::allocate_100_data_objects_box,
    mem::linear_allocator_benchmarks::allocate_100_data_objects_linear,
    mem::linear_allocator_benchmarks::allocate_100_data_objects_stack,
    mem::linear_allocator_benchmarks::allocate_100_data_objects_de_stack,
    mem::linear_allocator_benchmarks::allocate_100_data_objects_pool,
    // Realm
    mem::linear_mem_realm_benchmarks::memory_realm_linear_100_objects_unsafe,
    // Vector
    containers::vector_benchmarks::vec_1000_without_cap,
    containers::vector_benchmarks::vec_1000_with_cap,
    containers::vector_benchmarks::vec_1000_iteration,
    containers::vector_benchmarks::vec_1000_erase_range,
    // HandleMap
    containers::handlemap_benchmarks::handlemap_1000_insertion,
    containers::handlemap_benchmarks::handlemap_1000_iteration,
    containers::handlemap_benchmarks::handlemap_1000_remove,
];