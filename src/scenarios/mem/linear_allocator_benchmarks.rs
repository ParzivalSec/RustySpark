use std::mem;
use std::ptr;

use mem::allocators::base::*;
use mem::allocators::linear_allocator::{ LinearAllocator };
use mem::allocators::stack_allocator::{ StackAllocator };
use mem::allocators::double_ended_stack_allocator::{ DoubleEndedStackAllocator };
use mem::allocators::pool_allocator::{ PoolAllocator };

const LINEAR_OVERHEAD: usize = 4;
const STACK_OVERHEAD: usize = 8;

#[repr(C)]
#[derive(Default)]
struct AllocationData {
    pub data_block_1: [usize; 10],
    pub data_block_2: [usize; 10],
    pub data_block_3: [usize; 10],
    pub data_block_4: [usize; 10],
}

pub fn allocate_100_data_objects_box() {
    let mut allocations: Vec<Box<AllocationData>> = Vec::with_capacity(100);

    for _idx in 0 .. 100 {
        allocations.push(Box::new(AllocationData::default()));
    }
}

pub fn allocate_100_data_objects_linear() {
    unsafe {
        let mut allocations: [*mut AllocationData; 100] = mem::uninitialized();
        let linear_alloc = LinearAllocator::new(100 * (mem::size_of::<AllocationData>() + LINEAR_OVERHEAD));

        for idx in 0 .. 100 {
            allocations[idx] = linear_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. 100 {
            linear_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_100_data_objects_stack() {
    unsafe {
        let mut allocations: [*mut AllocationData; 100] = mem::uninitialized();
        let stack_alloc = StackAllocator::new(100 * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));

        for idx in 0 .. 100 {
            allocations[idx] = stack_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. 100 {
            stack_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_100_data_objects_de_stack() {
    unsafe {
        let mut allocations: [*mut AllocationData; 100] = mem::uninitialized();
        let de_stack_alloc = DoubleEndedStackAllocator::new(100 * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));

        for idx in 0 .. 100 {
            allocations[idx] = de_stack_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. 100 {
            de_stack_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_100_data_objects_pool() {
    unsafe {
        let mut allocations: [*mut AllocationData; 100] = mem::uninitialized();
        let pool_alloc = PoolAllocator::new(mem::size_of::<AllocationData>(), 100, 1, 0);

        for idx in 0 .. 100 {
            allocations[idx] = pool_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. 100 {
            pool_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

// SAFE ALLOCATIONS
// TODO: Add at the end of the test suite

pub fn allocate_100_data_objects_linear_safe() {
    unsafe {
        let linear_alloc = LinearAllocator::new(100 * (mem::size_of::<AllocationData>() + LINEAR_OVERHEAD));
        let mut allocations: [AllocatorBox<AllocationData, LinearAllocator>; 100] = mem::uninitialized();

        for idx in 0 .. 100 {
            allocations[idx] = linear_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_100_data_objects_stack_safe() {
    unsafe {
        let stack_alloc = StackAllocator::new(100 * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));       
        let mut allocations: [AllocatorBox<AllocationData, StackAllocator>; 100] = mem::uninitialized();
        

        for idx in 0 .. 100 {
            allocations[idx] = stack_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_100_data_objects_de_stack_safe() {
    unsafe {
        let de_stack_alloc = DoubleEndedStackAllocator::new(100 * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));
        let mut allocations: [AllocatorBox<AllocationData, DoubleEndedStackAllocator>; 100] = mem::uninitialized();
    
        for idx in 0 .. 100 {
            allocations[idx] = de_stack_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_100_data_objects_pool_safe() {
    unsafe {
        let pool_alloc = PoolAllocator::new(mem::size_of::<AllocationData>(), 100, 1, 0);
        let mut allocations: [AllocatorBox<AllocationData, PoolAllocator>; 100] = mem::uninitialized();

        for idx in 0 .. 100 {
            allocations[idx] = pool_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}