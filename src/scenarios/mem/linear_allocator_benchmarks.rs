use mem::allocators::base::*;
use mem::allocators::linear_allocator::{ LinearAllocator };
use mem::allocators::pool_allocator::{ PoolAllocator };

use libc;
use std;
use time;

use std::heap::{ Alloc };

const KB: usize = 1024;
const MB: usize = KB * 1024;

// Raw allocations
pub fn allocate_100_kb_hundred_times_raw() {
    let linear_alloc = LinearAllocator::new(400 + 100 * KB * 100);

    for _ in 0 .. 100 {
        let raw = linear_alloc.alloc_raw(100 * KB, 1, 0).unwrap();
        unsafe { libc::memset(raw.ptr as *mut libc::c_void, 0, 100 * KB); }
    }
}

pub fn allocate_1_mb_hundred_times_raw() {
    let linear_alloc = LinearAllocator::new(400 + 100 * MB);

    for _ in 0 .. 100 {
        let raw = linear_alloc.alloc_raw(MB, 1, 0).unwrap();
        unsafe { libc::memset(raw.ptr as *mut libc::c_void, 0, MB); }
    }
}

pub fn allocate_100_mb_ten_times_raw() {
    let linear_alloc = LinearAllocator::new(400 + 100 * MB * 10);

    for _ in 0 .. 10 {
        let raw = linear_alloc.alloc_raw(100 * MB, 1, 0).unwrap();
        unsafe { libc::memset(raw.ptr as *mut libc::c_void, 0, 100 * MB); }
    }
}

// Safe allocation of objects

#[repr(C)]
struct data_package_large {
    pub unique_id: usize,
    pub buffer: [u8; 100 * KB],
}

pub fn allocate_200_large_objects_with_box() {
    let mut objects: Vec<Box<data_package_large>> = Vec::with_capacity(200);

    for idx in 0 .. 200 {
        let raw = Box::new(data_package_large { 
                unique_id: 0, 
                buffer: unsafe { std::mem::uninitialized() }, 
            });

        objects.push(raw);
        objects[idx].unique_id = 0xDEADBEEF;
    }

    for idx in 0 .. 200 {
        objects[idx].unique_id += 10;
    }
}

pub fn allocate_200_large_objects_safe_pooled() {
    let pool_alloc = PoolAllocator::new(
        std::mem::size_of::<data_package_large>(),
        200,
        1,
        0
    );

    let mut objects: Vec<AllocatorBox<data_package_large, PoolAllocator>> = Vec::with_capacity(200);

    for idx in 0 .. 200 {
        let raw = pool_alloc.alloc(data_package_large { 
                unique_id: 0, 
                buffer: unsafe { std::mem::uninitialized() }, 
            }, 1, 0)
            .unwrap();


        objects.push(raw);
        objects[idx].unique_id = 0xDEADBEEF;
    }

    for idx in 0 .. 200 {
        objects[idx].unique_id += 10;
    }
}

pub fn heap() {
    for idx in 0 .. 200 {
        let layout = std::heap::Layout::from_size_align(50 * MB, 1).unwrap();
        let raw = unsafe { std::heap::Heap.alloc(layout).unwrap() };
        unsafe { libc::memset(raw as *mut libc::c_void, 0, 50 * MB); }
    }
}