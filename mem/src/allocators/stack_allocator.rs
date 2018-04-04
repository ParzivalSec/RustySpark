use std;
use std::marker::PhantomData;
use std::cell::RefCell;

use super::super::{ virtual_mem, pointer_util };
use super::allocator::{ Allocator, AllocatorMem };

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
/// The StakcAllocatorStorage is a type that is used to
/// expose a safe API for user allocations where the Allocator
/// itself is just mutating the Storage backing it
///
struct StackAllocatorStorage {
    pub use_internal_mem:   bool,
    pub mem_begin:          *mut u8,
    pub mem_end:            *mut u8,
    pub current_ptr:        *mut u8,
    #[cfg(stack_alloc_lifo_check)]
    pub allocation_id:      u32,
}

impl StackAllocatorStorage {
    ///
    /// Creates a new stack allocator storage and allocates the memory
    /// block requested by the allocator from the virtual memory API
    ///
    fn new(size: usize) -> StackAllocatorStorage {

        let virtual_mem = match virtual_mem::reserve_address_space(size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space = match virtual_mem::commit_physical_memory(virtual_mem, size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        StackAllocatorStorage {
            use_internal_mem: true,
            mem_begin: physical_address_space,
            mem_end: unsafe { physical_address_space.offset(size as isize) },
            current_ptr: physical_address_space,
            #[cfg(stack_alloc_lifo_check)]
            allocation_id: 0,
        }
    }
}

pub struct StackAllocator {
    storage: RefCell<StackAllocatorStorage>,
}

impl StackAllocator {
    pub fn new(size: usize) -> StackAllocator {
        debug_assert!(size > 0usize, "Size is not allowed to be 0");

        StackAllocator {
            storage: RefCell::new(StackAllocatorStorage::new(size)),
        }
    }
}

impl Allocator for StackAllocator {
    fn alloc(&self, size: usize, alignment: usize, offset: usize) -> Option<AllocatorMem> {
        debug_assert!(pointer_util::is_pot(alignment), "Alignment needs to be a power of two");

        let mut allocator_storage = self.storage.borrow_mut();
        let current_ptr_offset = allocator_storage.current_ptr as usize - allocator_storage.mem_begin as usize;
        let offset_before_alignment = offset + ALLOCATION_META_SIZE;

        unsafe {
            // Before aligning the pointer we need to offset it by offset + meta size to
            // properly align the pointer the user receives
            allocator_storage.current_ptr = allocator_storage.current_ptr.offset(offset_before_alignment as isize);
            allocator_storage.current_ptr = pointer_util::align_top(allocator_storage.current_ptr, alignment) as *mut u8;

            // If we overflow we cannot fulfill this allocation and return None
            let allocation_overflows = allocator_storage.current_ptr.offset(size as isize) > allocator_storage.mem_end;
            if  allocation_overflows {
                return None;
            }

            allocator_storage.current_ptr = allocator_storage.current_ptr.offset(-(offset_before_alignment as isize));

            let mut user_ptr = allocator_storage.current_ptr;
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
            allocator_storage.current_ptr = allocator_storage.current_ptr.offset((size + ALLOCATION_META_SIZE) as isize);

            Some(
                AllocatorMem {
                    ptr: user_ptr,
                    _phantom_slice: PhantomData,
                }
            )
        }
    }

    fn dealloc(&self, memory: AllocatorMem) {
        let raw_mem = memory.ptr;

        unsafe {
            let mut storage = self.storage.borrow_mut();
            let alloc_header = &mut *(raw_mem.offset(-(ALLOCATION_META_SIZE as isize)) as *mut AllocationHeader);
            

            #[cfg(stack_alloc_lifo_check)]
            {
                let was_freed_in_lifo_fashion = alloc_header.allocation_id == storage.allocation_id;
                debug_assert!(was_freed_in_lifo_fashion, "Stack allocator does only support LIFO fashioned freeing");
                storage.allocation_id -= 1;
            }

            storage.current_ptr = storage.mem_begin.offset(alloc_header.allocation_offset as isize);
        }
    }

    fn reset(&self) {
        let mut storage = self.storage.borrow_mut();
        storage.current_ptr = storage.mem_begin;
        #[cfg(stack_alloc_lifo_check)]
        {
            storage.allocation_id = 0;
        }
    }

    fn get_allocation_size(&self, memory: &AllocatorMem) -> usize {
        let alloc_header: &mut AllocationHeader;

        unsafe {
            let alloc_header_ptr: *const u32 = memory.ptr.offset(-(ALLOCATION_META_SIZE as isize)) as *const u32;
            alloc_header = &mut *(alloc_header_ptr as *mut AllocationHeader);
        }

        alloc_header.allocation_size as usize
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    #[test]
    fn single_allocation() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem = stack_allocator.alloc(256, 1, 0);
        assert!(raw_mem.is_some());
    }

    #[test]
    fn single_allocation_aligned() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem = stack_allocator.alloc(256, 16, 0);
        assert!(raw_mem.is_some());
        assert!(pointer_util::is_aligned_to(raw_mem.unwrap().ptr, 16));
    }

    #[test]
    fn single_allocation_aligned_with_offset() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem = stack_allocator.alloc(MB + 8, 16, 4);
        assert!(raw_mem.is_some());
        let ptr = raw_mem.unwrap().ptr;
        assert!(!pointer_util::is_aligned_to(ptr, 16), "Pointer without offset applied was already aligned");
        let offsetted_ptr = unsafe { ptr.offset(4) };
        assert!(pointer_util::is_aligned_to(offsetted_ptr, 16), "User pointer was not properly aligned");
    }

    #[test]
    fn multiple_allocations() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem_0 = stack_allocator.alloc(1 * MB, 1, 0);
        assert!(raw_mem_0.is_some());
        let raw_mem_1 = stack_allocator.alloc(1 * MB, 1, 0);
        assert!(raw_mem_1.is_some());
        let raw_mem_2 = stack_allocator.alloc(1 * MB, 1, 0);
        assert!(raw_mem_2.is_some());
        let raw_mem_3 = stack_allocator.alloc(1 * MB, 1, 0);
        assert!(raw_mem_3.is_some());
    }

    #[test]
    fn returns_none_on_oom() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem_0 = stack_allocator.alloc(6 * MB, 1, 0);
        assert!(raw_mem_0.is_some());
        let raw_mem_1 = stack_allocator.alloc(6 * MB, 1, 0);
        assert!(raw_mem_1.is_none());
    }

    #[test]
    fn deallocate_mem() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let raw_mem_0 = stack_allocator.alloc(256, 1, 0).unwrap();

        unsafe { std::ptr::write(raw_mem_0.ptr as *mut u32, 0xDEADBEEF) };
        stack_allocator.dealloc(raw_mem_0);

        let raw_mem_1 = stack_allocator.alloc(256, 1, 0).unwrap();
        let marker = unsafe { std::ptr::read(raw_mem_1.ptr as *mut u32) };

        assert!(marker == 0xDEADBEEF, "Previously placed marker was not there after deallocation");
    }

    #[test]
    fn reset_whole_allocator() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let mem_raw_0 = stack_allocator.alloc(MB, 4, 0).unwrap();
        stack_allocator.reset();
        let mem_raw_1 = stack_allocator.alloc(MB, 4, 0).unwrap();
        assert_eq!(mem_raw_0.ptr, mem_raw_1.ptr);
    }

    #[test]
    fn return_right_allocation_size() {
        let stack_allocator = StackAllocator::new(10 * MB);
        let mem_raw_0 = stack_allocator.alloc(MB * 2, 1, 0).unwrap();
        assert_eq!(stack_allocator.get_allocation_size(&mem_raw_0) == MB * 2, true);
        let mem_raw_1 = stack_allocator.alloc(MB * 3, 1, 0).unwrap();
        assert_eq!(stack_allocator.get_allocation_size(&mem_raw_1) == MB * 3, true);
        let mem_raw_2 = stack_allocator.alloc(MB * 4, 1, 0).unwrap();
        assert_eq!(stack_allocator.get_allocation_size(&mem_raw_2) == MB * 4, true);
    }

    #[test]
    fn allocation_do_not_invalidate_prev_ones() {
        struct SomeData {
            pub pos: usize,
            pub vel: usize,
        }

        let stack_allocator = StackAllocator::new(10 * MB);
        let mem_0 = stack_allocator.alloc(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_0 = unsafe { &mut *(mem_0.ptr as *mut SomeData) };

        data_ref_0.pos = 101;
        data_ref_0.vel = 111;

        let mem_1 = stack_allocator.alloc(std::mem::size_of::<SomeData>(), 1, 0).unwrap();
        let data_ref_1 = unsafe { &mut *(mem_1.ptr as *mut SomeData) };

        data_ref_1.pos = 202;
        data_ref_1.vel = 222;

        assert_eq!(data_ref_0.pos, 101);
        assert_eq!(data_ref_0.vel, 111);
        assert_eq!(data_ref_1.pos, 202);
        assert_eq!(data_ref_1.vel, 222);
    }

}