use std;
use std::cell::RefCell;
use spark_core::pointer_util;

use super::super::virtual_mem;
use super::base::{ Allocator, MemoryBlock, BasicAllocator };

///
/// The AllocationHeader struct describes meta-data
/// the allocator needs to store alongside of the 
/// allocations.
///
struct AllocationHeader {
    pub allocation_offset:  u32,
    pub allocation_size:    u32,
    #[cfg(stack_alloc_lifo_check)]
    pub allocation_id:      u32,
}

const ALLOCATION_META_SIZE: usize = std::mem::size_of::<AllocationHeader>();

///
/// The DoubleEndedStackAllocatorStorage is a type that is used to
/// expose a safe API for user allocations where the Allocator
/// itself is just mutating the Storage backing it
///
struct DoubleEndedStackAllocatorStorage {
    pub use_internal_mem:       bool,
    pub mem_begin:              *mut u8,
    pub mem_end:                *mut u8,
    pub current_front_ptr:      *mut u8,
    pub current_end_ptr:        *mut u8,
    #[cfg(stack_alloc_lifo_check)]
    pub front_allocation_id:    u32,
    #[cfg(stack_alloc_lifo_check)]
    pub back_allocation_id:     u32,
}

impl DoubleEndedStackAllocatorStorage {
    ///
    /// Creates a new stack allocator storage and allocates the memory
    /// block requested by the allocator from the virtual memory API
    ///
    fn new(size: usize) -> DoubleEndedStackAllocatorStorage {

        let virtual_mem = match virtual_mem::reserve_address_space(size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space = match virtual_mem::commit_physical_memory(virtual_mem, size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space_end =  unsafe { physical_address_space.offset(size as isize) };

        DoubleEndedStackAllocatorStorage {
            use_internal_mem:       true,
            mem_begin:              physical_address_space,
            mem_end:                physical_address_space_end,
            current_front_ptr:      physical_address_space,
            current_end_ptr:        physical_address_space_end,
            #[cfg(stack_alloc_lifo_check)]
            front_allocation_id:    0,
            #[cfg(stack_alloc_lifo_check)]
            back_allocation_id:     0,
        }
    }
}

pub struct DoubleEndedStackAllocator {
    storage: RefCell<DoubleEndedStackAllocatorStorage>,
}

impl DoubleEndedStackAllocator {
    pub fn alloc_raw_back(&self, size: usize, alignment: usize, offset: usize) -> Option<MemoryBlock> {
       debug_assert!(pointer_util::is_pot(alignment), "Alignment needs to be a power of two");

        let mut allocator_storage = self.storage.borrow_mut();
        let current_ptr_offset = allocator_storage.current_end_ptr as isize - allocator_storage.mem_end as isize;
        let offset_before_alignment = offset + ALLOCATION_META_SIZE;

        unsafe {
            allocator_storage.current_end_ptr = allocator_storage.current_end_ptr.offset(-(size as isize));
            allocator_storage.current_end_ptr = pointer_util::align_bottom(allocator_storage.current_end_ptr, alignment) as *mut u8;

            // If we overflow we cannot fulfill this allocation and return None
            let allocation_overflows_front_block = allocator_storage.current_end_ptr.offset(-(offset_before_alignment as isize)) < allocator_storage.current_front_ptr;
            if  allocation_overflows_front_block {
                return None;
            }

            allocator_storage.current_end_ptr = allocator_storage.current_end_ptr.offset(-(offset_before_alignment as isize));

            let mut user_ptr = allocator_storage.current_end_ptr;
            let as_alloc_header = &mut *(user_ptr as *mut AllocationHeader);

            // Write allocation meta data
            as_alloc_header.allocation_offset = current_ptr_offset as u32;
            as_alloc_header.allocation_size = size as u32;
            #[cfg(stack_alloc_lifo_check)]
            {
                allocator_storage.back_allocation_id += 1;
                as_alloc_header.allocation_id = allocator_storage.back_allocation_id;
            }

            user_ptr = user_ptr.offset(ALLOCATION_META_SIZE as isize);
            allocator_storage.current_end_ptr = allocator_storage.current_end_ptr.offset(-(ALLOCATION_META_SIZE as isize));

            Some(MemoryBlock::new(user_ptr))
        }
    }

    pub fn dealloc_raw_back(&self, memory: MemoryBlock) {
       let raw_mem = memory.ptr;

        unsafe {
            let mut storage = self.storage.borrow_mut();
            let alloc_header = &mut *(raw_mem.offset(-(ALLOCATION_META_SIZE as isize)) as *mut AllocationHeader);
            
            {
                let ptr_in_range = raw_mem >= storage.mem_begin && raw_mem < storage.mem_end;
                debug_assert!(ptr_in_range, "AllocatorMem was not allocated by this allocator");
                let ptr_in_back_block = raw_mem >= storage.current_end_ptr;
                debug_assert!(ptr_in_back_block, "AllocatorMem was not allocated via `alloc_back` (back block)");
            }

            #[cfg(stack_alloc_lifo_check)]
            {
                let was_freed_in_lifo_fashion = alloc_header.allocation_id == storage.back_allocation_id;
                debug_assert!(was_freed_in_lifo_fashion, "Double ended stack allocator does only support LIFO fashioned freeing");
                storage.back_allocation_id -= 1;
            }

            storage.current_end_ptr = storage.mem_end.offset(-(alloc_header.allocation_offset as isize));
        }
    }
}

impl BasicAllocator for DoubleEndedStackAllocator {
    type AllocatorImplementation = DoubleEndedStackAllocator;

    fn new(size: usize) -> Self::AllocatorImplementation {
        debug_assert!(size > 0usize, "Size is not allowed to be 0");

        DoubleEndedStackAllocator {
            storage: RefCell::new(DoubleEndedStackAllocatorStorage::new(size)),
        }
    }
}

impl Allocator for DoubleEndedStackAllocator {
    
    fn alloc_raw(&self, size: usize, alignment: usize, offset: usize) -> Option<MemoryBlock> {
        debug_assert!(pointer_util::is_pot(alignment), "Alignment needs to be a power of two");

        let mut allocator_storage = self.storage.borrow_mut();
        let current_ptr_offset = allocator_storage.current_front_ptr as usize - allocator_storage.mem_begin as usize;
        let offset_before_alignment = offset + ALLOCATION_META_SIZE;

        unsafe {
            // Before aligning the pointer we need to offset it by offset + meta size to
            // properly align the pointer the user receives
            allocator_storage.current_front_ptr = allocator_storage.current_front_ptr.offset(offset_before_alignment as isize);
            allocator_storage.current_front_ptr = pointer_util::align_top(allocator_storage.current_front_ptr, alignment) as *mut u8;

            // If we overflow we cannot fulfill this allocation and return None
            let allocation_overflows_end_block = allocator_storage.current_front_ptr.offset((size - offset) as isize) > allocator_storage.current_end_ptr;
            if  allocation_overflows_end_block {
                return None;
            }

            allocator_storage.current_front_ptr = allocator_storage.current_front_ptr.offset(-(offset_before_alignment as isize));

            let mut user_ptr = allocator_storage.current_front_ptr;
            let as_alloc_header = &mut *(user_ptr as *mut AllocationHeader);

            // Write allocation meta data
            as_alloc_header.allocation_offset = current_ptr_offset as u32;
            as_alloc_header.allocation_size = size as u32;
            #[cfg(stack_alloc_lifo_check)]
            {
                allocator_storage.allocation_id += 1;
                as_alloc_header.allocation_id = allocator_storage.allocation_id;
            }

            user_ptr = user_ptr.offset(ALLOCATION_META_SIZE as isize);
            allocator_storage.current_front_ptr = allocator_storage.current_front_ptr.offset((size + ALLOCATION_META_SIZE) as isize);

            Some(MemoryBlock::new(user_ptr))
        }
    }

    fn dealloc_raw(&self, memory: MemoryBlock) {
        let raw_mem = memory.ptr;

        unsafe {
            let mut storage = self.storage.borrow_mut();
            let alloc_header = &mut *(raw_mem.offset(-(ALLOCATION_META_SIZE as isize)) as *mut AllocationHeader);
            
            {
                let ptr_in_range = raw_mem >= storage.mem_begin && raw_mem < storage.mem_end;
                debug_assert!(ptr_in_range, "AllocatorMem was not allocated by this allocator");
                let ptr_in_front_block = raw_mem < storage.current_end_ptr;
                debug_assert!(ptr_in_front_block, "AllocatorMem was not allocated via `alloc` (front block)");
            }

            #[cfg(stack_alloc_lifo_check)]
            {
                let was_freed_in_lifo_fashion = alloc_header.allocation_id == storage.front_allocation_id;
                debug_assert!(was_freed_in_lifo_fashion, "Double ended stack allocator does only support LIFO fashioned freeing");
                storage.front_allocation_id -= 1;
            }

            storage.current_front_ptr = storage.mem_begin.offset(alloc_header.allocation_offset as isize);
        }
    }

    fn reset(&self) {
        let mut storage = self.storage.borrow_mut();

        storage.current_front_ptr = storage.mem_begin;
        storage.current_end_ptr = storage.mem_end;
        
        #[cfg(stack_alloc_lifo_check)]
        {
            storage.front_allocation_id = 0;
            storage.back_allocation_id = 0;
        }
    }

    fn get_allocation_size(&self, memory: &MemoryBlock) -> usize {
        let alloc_header: &mut AllocationHeader;

        unsafe {
            let alloc_header_ptr: *const u32 = memory.ptr.offset(-(ALLOCATION_META_SIZE as isize)) as *const u32;
            alloc_header = &mut *(alloc_header_ptr as *mut AllocationHeader);
        }

        alloc_header.allocation_size as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    #[test]
    fn single_allocation_front() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem = de_stack_alloc.alloc_raw(MB, 1, 0);
        assert!(mem.is_some());
    }

    #[test]
    fn single_allocation_back() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem = de_stack_alloc.alloc_raw_back(MB, 1, 0);
        assert!(mem.is_some());
    }

    #[test]
    fn single_allocation_front_aligned() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem = de_stack_alloc.alloc_raw(MB, 16, 0);
        assert!(mem.is_some());
        assert!(pointer_util::is_aligned_to(mem.unwrap().ptr, 16));
    }

    #[test]
    fn single_allocation_front_aligned_with_offset() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let raw_mem = de_stack_alloc.alloc_raw(MB + 8, 16, 4);
        assert!(raw_mem.is_some());
        let ptr = raw_mem.unwrap().ptr;
        assert!(!pointer_util::is_aligned_to(ptr, 16), "Pointer without offset applied was already aligned");
        let offsetted_ptr = unsafe { ptr.offset(4) };
        assert!(pointer_util::is_aligned_to(offsetted_ptr, 16), "User pointer was not properly aligned");
    }

    #[test]
    fn single_allocation_back_aligned() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem = de_stack_alloc.alloc_raw_back(MB, 16, 0);
        assert!(mem.is_some());
        assert!(pointer_util::is_aligned_to(mem.unwrap().ptr, 16));
    }

    #[test]
    fn single_allocation_back_aligned_with_offset() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let raw_mem = de_stack_alloc.alloc_raw_back(MB + 8, 16, 4);
        assert!(raw_mem.is_some());
        let ptr = raw_mem.unwrap().ptr;
        assert!(!pointer_util::is_aligned_to(ptr, 16), "Pointer without offset applied was already aligned");
        let offsetted_ptr = unsafe { ptr.offset(4) };
        assert!(pointer_util::is_aligned_to(offsetted_ptr, 16), "User pointer was not properly aligned");
    }

    #[test]
    fn multiple_allocations_front() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw(MB, 1, 0);
        assert!(mem_0.is_some());
        let mem_1 = de_stack_alloc.alloc_raw(MB, 1, 0);
        assert!(mem_1.is_some());
        let mem_2 = de_stack_alloc.alloc_raw(MB, 1, 0);
        assert!(mem_2.is_some());
        let mem_3 = de_stack_alloc.alloc_raw(MB, 1, 0);
        assert!(mem_3.is_some());
    }

    #[test]
    fn multiple_allocations_back() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw_back(MB, 1, 0);
        assert!(mem_0.is_some());
        let mem_1 = de_stack_alloc.alloc_raw_back(MB, 1, 0);
        assert!(mem_1.is_some());
        let mem_2 = de_stack_alloc.alloc_raw_back(MB, 1, 0);
        assert!(mem_2.is_some());
        let mem_3 = de_stack_alloc.alloc_raw_back(MB, 1, 0);
        assert!(mem_3.is_some());
    }

    #[test]
    fn dealloc_front() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw(MB, 1, 0).unwrap();
        
        unsafe { std::ptr::write(mem_0.ptr as *mut u32, 0xDEADBEEF) };
        de_stack_alloc.dealloc_raw(mem_0);

        let mem_1 = de_stack_alloc.alloc_raw(MB, 1, 0).unwrap();
        let marker = unsafe { std::ptr::read(mem_1.ptr as *mut u32) };

        assert!(marker == 0xDEADBEEF, "Previously placed marker was not there after deallocation");
    }

    #[test]
    fn dealloc_back() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw_back(MB, 1, 0).unwrap();
        
        unsafe { std::ptr::write(mem_0.ptr as *mut u32, 0xDEADBEEF) };
        de_stack_alloc.dealloc_raw_back(mem_0);

        let mem_1 = de_stack_alloc.alloc_raw_back(MB, 1, 0).unwrap();
        let marker = unsafe { std::ptr::read(mem_1.ptr as *mut u32) };

        assert!(marker == 0xDEADBEEF, "Previously placed marker was not there after deallocation");
    }
    
    #[test]
    #[should_panic(expected = "AllocatorMem was not allocated via `alloc` (front block)")]
    fn assert_wrong_front_deallocation() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw_back(MB, 1, 0).unwrap();
        de_stack_alloc.dealloc_raw(mem_0);
    }

    #[test]
    #[should_panic(expected = "AllocatorMem was not allocated via `alloc_back` (back block)")]
    fn assert_wrong_back_deallocation() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw(MB, 1, 0).unwrap();
        de_stack_alloc.dealloc_raw_back(mem_0);
    }

    #[test]
    fn return_none_on_back_overlap() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let _mem_back = de_stack_alloc.alloc_raw_back(6 * MB, 1, 0);
        let mem_front = de_stack_alloc.alloc_raw(6 * MB, 1, 0);
        assert!(mem_front.is_none());
    }

    #[test]
    fn return_none_on_front_overlap() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let _mem_front = de_stack_alloc.alloc_raw(6 * MB, 1, 0);
        let mem_back = de_stack_alloc.alloc_raw_back(6 * MB, 1, 0);
        assert!(mem_back.is_none());
    }

    #[test]
    fn reset_whole_allocator() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_front_0 = de_stack_alloc.alloc_raw(MB, 4, 0).unwrap();
        let mem_back_0 = de_stack_alloc.alloc_raw_back(MB, 4, 0).unwrap();
        de_stack_alloc.reset();
        let mem_front_1 = de_stack_alloc.alloc_raw(MB, 4, 0).unwrap();
        assert_eq!(mem_front_0.ptr, mem_front_1.ptr);
        let mem_back_1 = de_stack_alloc.alloc_raw_back(MB, 4, 0).unwrap();
        assert_eq!(mem_back_0.ptr, mem_back_1.ptr);
    }

    #[test]
    fn get_right_allocation_size() {
        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_raw_0 = de_stack_alloc.alloc_raw(MB * 2, 1, 0).unwrap();
        assert_eq!(de_stack_alloc.get_allocation_size(&mem_raw_0) == MB * 2, true);
        let mem_raw_1 = de_stack_alloc.alloc_raw(MB * 3, 1, 0).unwrap();
        assert_eq!(de_stack_alloc.get_allocation_size(&mem_raw_1) == MB * 3, true);
        let mem_raw_2 = de_stack_alloc.alloc_raw(MB * 4, 1, 0).unwrap();
        assert_eq!(de_stack_alloc.get_allocation_size(&mem_raw_2) == MB * 4, true);
    }

    #[test]
    fn front_allocation_do_not_invalidate_prev_ones() {
        struct SomeData {
            pub pos: usize,
            pub vel: usize,
        }

        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_0 = unsafe { &mut *(mem_0.ptr as *mut SomeData) };

        data_ref_0.pos = 101;
        data_ref_0.vel = 111;

        let mem_1 = de_stack_alloc.alloc_raw(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_1 = unsafe { &mut *(mem_1.ptr as *mut SomeData) };

        data_ref_1.pos = 202;
        data_ref_1.vel = 222;

        assert_eq!(data_ref_0.pos, 101);
        assert_eq!(data_ref_0.vel, 111);
        assert_eq!(data_ref_1.pos, 202);
        assert_eq!(data_ref_1.vel, 222);
    }

    #[test]
    fn back_allocation_do_not_invalidate_prev_ones() {
        struct SomeData {
            pub pos: usize,
            pub vel: usize,
        }

        let de_stack_alloc = DoubleEndedStackAllocator::new(10 * MB);
        let mem_0 = de_stack_alloc.alloc_raw_back(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_0 = unsafe { &mut *(mem_0.ptr as *mut SomeData) };

        data_ref_0.pos = 101;
        data_ref_0.vel = 111;

        let mem_1 = de_stack_alloc.alloc_raw_back(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_1 = unsafe { &mut *(mem_1.ptr as *mut SomeData) };

        data_ref_1.pos = 202;
        data_ref_1.vel = 222;

        assert_eq!(data_ref_0.pos, 101);
        assert_eq!(data_ref_0.vel, 111);
        assert_eq!(data_ref_1.pos, 202);
        assert_eq!(data_ref_1.vel, 222);
    }
}