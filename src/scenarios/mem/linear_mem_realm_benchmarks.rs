use std::mem;
use std::ptr;

use mem::allocators::base::*;
use mem::allocators::linear_allocator::{ LinearAllocator };
use mem::bounds_checker::simple_bounds_checker::SimpleBoundsChecker;
use mem::memory_realm::basic_realm::BasicMemoryRealm;

type LinearBoundsCheckedRealm = BasicMemoryRealm<LinearAllocator, SimpleBoundsChecker>;

const LINEAR_OVERHEAD: usize = 4;
const CANARY_OVERHEAD: usize = 8;

#[repr(C)]
#[derive(Default)]
struct AllocationData {
    pub data_block_1: [usize; 10],
    pub data_block_2: [usize; 10],
    pub data_block_3: [usize; 10],
    pub data_block_4: [usize; 10],
}

pub fn memory_realm_linear_100_objects_unsafe() {
    unsafe {
        let mut allocations: [*mut AllocationData; 100] = mem::uninitialized();
        let realm = LinearBoundsCheckedRealm::new(100 * (mem::size_of::<AllocationData>() + LINEAR_OVERHEAD + CANARY_OVERHEAD));

        for idx in 0 .. 100 {
            allocations[idx] = realm.alloc(mem::size_of::<AllocationData>(), 1).unwrap().ptr as *mut AllocationData;
            ptr::write(allocations[idx], AllocationData::default());
        }

        for idx in 0 .. 100 {
            realm.dealloc(MemoryBlock::new(allocations[idx] as *mut u8));
        }
    }
}