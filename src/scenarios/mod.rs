pub mod mem;

pub type BenchmarkFunction = fn();

pub static SCENARIOS: &'static [BenchmarkFunction] = &[
    mem::linear_allocator_benchmarks::allocate_100_kb_hundred_times_raw,
    mem::linear_allocator_benchmarks::allocate_1_mb_hundred_times_raw,
    mem::linear_allocator_benchmarks::allocate_100_mb_ten_times_raw,
    mem::linear_allocator_benchmarks::allocate_200_large_objects_with_box,
    mem::linear_allocator_benchmarks::allocate_200_large_objects_safe_pooled,
    mem::linear_allocator_benchmarks::heap,
];