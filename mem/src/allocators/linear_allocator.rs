use std;
use std::cell::RefCell;

use spark_core::pointer_util;
use super::super::virtual_mem;
use super::base::{ Allocator, MemoryBlock, BasicAllocator };


///
/// The LinearAllocatorStorage is a type that is used to
/// expose a safe API for user allocations where the Allocator
/// itself is just mutating the Storage backing it
///
struct LinearAllocatorStorage {
    pub use_internal_mem:   bool,
    pub mem_begin:          *mut u8,
    pub mem_end:            *mut u8,
    pub current_ptr:        *mut u8,
}

///
/// The AllocationHeader struct describes meta-data
/// the allocator needs to store alongside of the 
/// allocations.
///
struct AllocationHeader {
    pub allocation_size: u32,
}

const ALLOCATION_META_SIZE: usize = std::mem::size_of::<AllocationHeader>();

impl LinearAllocatorStorage {
    ///
    /// Creates a new linear allocator storage and allocates the memory
    /// block requested by the allocator from the virtual memory API
    ///
    fn new(size: usize) -> LinearAllocatorStorage {

        let virtual_mem = match virtual_mem::reserve_address_space(size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space = match virtual_mem::commit_physical_memory(virtual_mem, size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        LinearAllocatorStorage {
            use_internal_mem: true,
            mem_begin: physical_address_space,
            mem_end: unsafe { physical_address_space.offset(size as isize) },
            current_ptr: physical_address_space,
        }
    }
}

///
/// LinearAllocator is the struct the user works with directly. Due to interior
/// mutability ensured by the RefCell wrapping the storage a user can issue several
/// allocations requests without freezing the allocator. The user does not loose
/// checks for dangling MemoryBlocks that would outlive the Allocator.
///
pub struct LinearAllocator {
    storage: RefCell<LinearAllocatorStorage>,
}

impl LinearAllocator {
    pub fn new(size: usize) -> LinearAllocator {
        debug_assert!(size > 0usize, "Size is not allowed to be 0");

        LinearAllocator {
            storage: RefCell::new(LinearAllocatorStorage::new(size)),
        }
    }
}

impl BasicAllocator for LinearAllocator {
    type AllocatorImplementation = LinearAllocator;

    fn new(size: usize) -> Self::AllocatorImplementation {
        debug_assert!(size > 0usize, "Size is not allowed to be 0");

        LinearAllocator {
            storage: RefCell::new(LinearAllocatorStorage::new(size)),
        }
    }
}

impl Allocator for LinearAllocator {
    ///
    /// `alloc` processes an allocation request issued by an user.
    /// The pointer contained in the returned MemoryBlock us guaranteed
    /// to be aligned to a byte boundary matching `alignment`. The offset
    /// can be used by the issuer to reserve some space for meta data right
    /// in front of the aligned pointer.
    ///
    fn alloc_raw(&self, size: usize, alignment: usize, offset: usize) 
        -> Option<MemoryBlock>
    {
        debug_assert!(pointer_util::is_pot(alignment), "Alignment needs to be a power of two");

        let mut allocator_storage = self.storage.borrow_mut();
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

            std::ptr::write(user_ptr as *mut u32, size as u32);
            user_ptr = user_ptr.offset(ALLOCATION_META_SIZE as isize);
            allocator_storage.current_ptr = allocator_storage.current_ptr.offset((size + ALLOCATION_META_SIZE) as isize);

            Some(MemoryBlock::new(user_ptr))
        }
    }

    ///
    /// `dealloc` yields a no-op in this LinearAllocator
    ///
    fn dealloc_raw(&self, _memory: MemoryBlock) {}

    ///
    /// To free issued allocations one has to call `reset` to return the
    /// allocator to its initial state. Be careful, at the moment this function
    /// does invalidate ALL user managed MemoryBlockBlocks, without any
    /// safety mechanism for the user holding it
    ///
    fn reset(&self) {
        let mut storage = self.storage.borrow_mut();
        storage.current_ptr = storage.mem_begin;
    }

    ///
    /// Returns the size of the allocation the MemoryBlockBlock refers to
    ///
    fn get_allocation_size(&self, memory: &MemoryBlock) -> usize
    {
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
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw = linear_alloc.alloc_raw(MB, 1, 0);
        assert!(mem_raw.is_some());
    }

    #[test]
    fn single_allocation_aligned() {
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw_aligned = linear_alloc.alloc_raw(MB, 16, 0);
        assert!(mem_raw_aligned.is_some());
        assert!(pointer_util::is_aligned_to(mem_raw_aligned.unwrap().ptr, 16));
    }

    #[test]
    fn single_allocation_aligned_with_offset() {
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw_aligned = linear_alloc.alloc_raw(MB + 8, 16, 4);
        assert!(mem_raw_aligned.is_some());
        let ptr = mem_raw_aligned.unwrap().ptr;
        assert!(!pointer_util::is_aligned_to(ptr, 16), "Pointer without offset applied was aligned");
        let offsetted_ptr = unsafe { ptr.offset(4) };
        assert!(pointer_util::is_aligned_to(offsetted_ptr, 16), "User pointer was not properly aligned");
    }

    #[test]
    fn multiple_allocations() {
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw_0 = linear_alloc.alloc_raw(MB, 4, 0);
        assert!(mem_raw_0.is_some());
        let mem_raw_1 = linear_alloc.alloc_raw(MB, 4, 0);
        assert!(mem_raw_1.is_some());
        let mem_raw_2 = linear_alloc.alloc_raw(MB, 4, 0);
        assert!(mem_raw_2.is_some());
    }

    #[test]
    fn reset_whole_allocator() {
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw_0 = linear_alloc.alloc_raw(MB, 4, 0).unwrap();
        linear_alloc.reset();
        let mem_raw_1 = linear_alloc.alloc_raw(MB, 4, 0).unwrap();
        assert_eq!(mem_raw_0.ptr, mem_raw_1.ptr);
    }

    #[test]
    fn return_right_allocation_size() {
        let linear_alloc: LinearAllocator = LinearAllocator::new(10 * MB);
        let mem_raw_0 = linear_alloc.alloc_raw(MB * 2, 1, 0).unwrap();
        assert_eq!(linear_alloc.get_allocation_size(&mem_raw_0) == MB * 2, true);
        let mem_raw_1 = linear_alloc.alloc_raw(MB * 3, 1, 0).unwrap();
        assert_eq!(linear_alloc.get_allocation_size(&mem_raw_1) == MB * 3, true);
        let mem_raw_2 = linear_alloc.alloc_raw(MB * 4, 1, 0).unwrap();
        assert_eq!(linear_alloc.get_allocation_size(&mem_raw_2) == MB * 4, true);
    }

    #[test]
    fn allocate_safely() {
        struct Data {
            pub result: f32,
            pub id: usize,
        }
        
        let linear_alloc: LinearAllocator = LinearAllocator::new(std::mem::size_of::<Data>() + 4);

        {
            let mut data_box = linear_alloc.alloc(Data { result: 1.0, id: 1 }, 1, 0).unwrap();
            let data = &mut(*data_box);

            assert_eq!(data.result, 1.0);
            assert_eq!(data.id, 1);

            data.result = 2.0;
            data.id = 2;

            assert_eq!(data.result, 2.0);
            assert_eq!(data.id, 2);
        }

        let data_box = linear_alloc.alloc(Data { result: 1.0, id: 1 }, 1, 0);
        assert!(data_box.is_none(), "Second allocation did not fail, LinearAllocator does not allow freeing hence should be OOM");
    }
}