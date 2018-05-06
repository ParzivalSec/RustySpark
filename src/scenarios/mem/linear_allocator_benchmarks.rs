use std::mem;
use std::ptr;

use mem::allocators::base::*;
use mem::allocators::linear_allocator::{ LinearAllocator };
use mem::allocators::stack_allocator::{ StackAllocator };
use mem::allocators::double_ended_stack_allocator::{ DoubleEndedStackAllocator };
use mem::allocators::pool_allocator::{ PoolAllocator };

const ALLOCATION_NUM: usize = 1_000;
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

pub fn allocate_1000_data_objects_box() {
    let mut allocations: Vec<Box<AllocationData>> = Vec::with_capacity(ALLOCATION_NUM);

    for _idx in 0 .. ALLOCATION_NUM {
        allocations.push(Box::new(AllocationData::default()));
    }
}

pub fn allocate_1000_data_objects_linear() {
    unsafe {
        let mut allocations: [*mut AllocationData; ALLOCATION_NUM] = mem::uninitialized();
        let linear_alloc = LinearAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + LINEAR_OVERHEAD));

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = linear_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. ALLOCATION_NUM {
            linear_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_1000_data_objects_stack() {
    unsafe {
        let mut allocations: [*mut AllocationData; ALLOCATION_NUM] = mem::uninitialized();
        let stack_alloc = StackAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = stack_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. ALLOCATION_NUM {
            stack_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_1000_data_objects_de_stack() {
    unsafe {
        let mut allocations: [*mut AllocationData; ALLOCATION_NUM] = mem::uninitialized();
        let de_stack_alloc = DoubleEndedStackAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = de_stack_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. ALLOCATION_NUM {
            de_stack_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

pub fn allocate_1000_data_objects_pool() {
    unsafe {
        let mut allocations: [*mut AllocationData; ALLOCATION_NUM] = mem::uninitialized();
        let pool_alloc = PoolAllocator::new(mem::size_of::<AllocationData>(), ALLOCATION_NUM, 1, 0);

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = pool_alloc.alloc_raw(mem::size_of::<AllocationData>(), 1, 0).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. ALLOCATION_NUM {
            pool_alloc.dealloc_raw(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}

// SAFE ALLOCATIONS
// TODO: Add at the end of the test suite

pub fn allocate_1000_data_objects_linear_safe() {
    unsafe {
        let linear_alloc = LinearAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + LINEAR_OVERHEAD));
        let mut allocations: [AllocatorBox<AllocationData, LinearAllocator>; ALLOCATION_NUM] = mem::uninitialized();

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = linear_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_1000_data_objects_stack_safe() {
    unsafe {
        let stack_alloc = StackAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));       
        let mut allocations: [AllocatorBox<AllocationData, StackAllocator>; ALLOCATION_NUM] = mem::uninitialized();
        

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = stack_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_1000_data_objects_de_stack_safe() {
    unsafe {
        let de_stack_alloc = DoubleEndedStackAllocator::new(ALLOCATION_NUM * (mem::size_of::<AllocationData>() + STACK_OVERHEAD));
        let mut allocations: [AllocatorBox<AllocationData, DoubleEndedStackAllocator>; ALLOCATION_NUM] = mem::uninitialized();
    
        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = de_stack_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}

pub fn allocate_1000_data_objects_pool_safe() {
    unsafe {
        let pool_alloc = PoolAllocator::new(mem::size_of::<AllocationData>(), ALLOCATION_NUM, 1, 0);
        let mut allocations: [AllocatorBox<AllocationData, PoolAllocator>; ALLOCATION_NUM] = mem::uninitialized();

        for idx in 0 .. ALLOCATION_NUM {
            allocations[idx] = pool_alloc.alloc(AllocationData::default(), 1, 0).unwrap();
        }
    }
}