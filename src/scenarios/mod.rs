pub mod mem;
pub mod containers;
pub mod ecs;

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
    // Ringbuffer
    containers::ringbuffer_benchmarks::ringbuffe_10000_write,
    containers::ringbuffer_benchmarks::ringbuffer_10000_read,
    containers::ringbuffer_benchmarks::ringbuffer_15000_write_wrapping,
    // ECS
    ecs::ecs_benchmarks::ecs_create_10000_with_pos,
    ecs::ecs_benchmarks::ecs_create_100000_with_pos,
    ecs::ecs_benchmarks::ecs_create_10000_with_pos_vel,
    ecs::ecs_benchmarks::ecs_create_100000_with_pos_vel,
    ecs::ecs_benchmarks::ecs_iterate_10000_pos,
    ecs::ecs_benchmarks::ecs_iterate_100000_pos,
    ecs::ecs_benchmarks::ecs_remove_5000_pos,
    ecs::ecs_benchmarks::ecs_remove_50000_pos,
];