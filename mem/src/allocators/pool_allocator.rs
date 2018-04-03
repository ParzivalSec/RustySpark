use std;
use std::marker::PhantomData;
use std::cell::RefCell;

use super::super::{ virtual_mem, pointer_util, freelist };
use super::allocator::{ Allocator, AllocatorMem };

struct PoolAllocatorStorage {
    pub use_internal_mem:       bool,
    pub mem_begin:              *mut u8,
    pub mem_end:                *mut u8,
    pub first_block_ptr:        *mut u8,
    pub max_element_size:       usize,
    pub max_element_alignment:  usize,
    pub min_block_size:         usize,
    pub free_list:              freelist::FreeList,
}

fn round_to_next_multiple(num: usize, multiple: usize) -> usize {
    let remainer = num % multiple;
    num - remainer + multiple * !!(remainer)
}

fn calculate_minimal_block_size(max_size: usize, max_alignment: usize) -> usize {
    if max_size < max_alignment {
        max_alignment
    }
    else {
        round_to_next_multiple(max_size, max_alignment)
    }
}

impl PoolAllocatorStorage {
    fn new(size: usize,
        min_block_size: usize,
        max_element_size: usize,
        max_element_alignment: usize,
        offset: usize
        ) -> PoolAllocatorStorage {
        
        let virtual_mem = match virtual_mem::reserve_address_space(size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space = match virtual_mem::commit_physical_memory(virtual_mem, size) {
            Some(address) => address,
            None => std::ptr::null_mut(),
        };

        let physical_address_space_end =  unsafe { physical_address_space.offset(size as isize) };
        
        let first_block_ptr = unsafe {
            let signed_offset = offset as isize;
            let mut aligned_ptr  = pointer_util::align_top(physical_address_space.offset(signed_offset), max_element_alignment) as *mut u8;
            aligned_ptr = aligned_ptr.offset(-signed_offset);

            aligned_ptr
        };

        PoolAllocatorStorage {
            use_internal_mem:   true,
            mem_begin:          physical_address_space,
            mem_end:            physical_address_space_end,
            first_block_ptr,
            max_element_size,
            max_element_alignment,
            min_block_size,
            free_list:          freelist::FreeList::new_from(first_block_ptr, physical_address_space_end, min_block_size),
        }
    }  
}

pub struct PoolAllocator {
    storage: RefCell<PoolAllocatorStorage>,
}

impl PoolAllocator {
    pub fn new(max_element_size: usize, element_count: usize, max_element_alignment: usize, offset: usize) -> PoolAllocator {
        let required_memory_size = (element_count * max_element_size) + max_element_alignment;

        PoolAllocator {
            storage: RefCell::new(PoolAllocatorStorage::new(
                required_memory_size,
                calculate_minimal_block_size(max_element_size, max_element_alignment),
                max_element_size,
                max_element_alignment,
                offset)
            ),
        }
    }
}

impl Allocator for PoolAllocator {
    fn alloc(&self, size: usize, alignment: usize, _offset: usize) -> Option<AllocatorMem> {
        let storage = self.storage.borrow_mut();

        {
            let size_lesser_or_equal_max_element_size = size <= storage.max_element_size;
            debug_assert!(size_lesser_or_equal_max_element_size, "Alloc size has to be less or equal max element size");
            let alignment_lesser_or_equal_max_element_alignment = alignment <= storage.max_element_alignment;
            debug_assert!(alignment_lesser_or_equal_max_element_alignment, "Alloc alignment has to be less or equal max element alignment");
        }
        
        let ptr = storage.free_list.get_block();
        
        if ptr.is_null() {
            return None;
        }

        Some(AllocatorMem {
            ptr,
            _phantom_slice: PhantomData,
        })
    }

    fn dealloc(&self, memory: AllocatorMem) {
        {
            // TODO: Asserts
        }

        let storage = self.storage.borrow_mut();
        storage.free_list.return_block(memory.ptr);
    }

    fn reset(&self) {
        let mut storage = self.storage.borrow_mut();
        storage.free_list = freelist::FreeList::new_from(
            storage.first_block_ptr,
            storage.mem_end,
            storage.min_block_size
        );
    }

    fn get_allocation_size(&self, _memory: &AllocatorMem) -> usize {
        let storage = self.storage.borrow_mut();
        storage.min_block_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    
    struct Particle {
        pub lifetime:   f32,
        pub speed:      usize,
    }

    #[test]
    fn single_allocation() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            1,
            0
        );

        let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
        assert!(obj_0.is_some());
    }

    #[test]
    fn single_allocation_aligned() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            16,
            0
        );

        let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_0.is_some());
        assert!(pointer_util::is_aligned_to(obj_0.unwrap().ptr, 16));
    }

    #[test]
    fn multiple_allocations() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            1,
            0
        );

        let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
        assert!(obj_0.is_some());
        let obj_1 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
        assert!(obj_1.is_some());
        let obj_2 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
        assert!(obj_2.is_some());
        let obj_3 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
        assert!(obj_3.is_some());
    }

    #[test]
    fn multiple_allocations_aligned() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            16,
            0
        );

        let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_0.is_some());
        assert!(pointer_util::is_aligned_to(obj_0.unwrap().ptr, 16));
        let obj_1 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_1.is_some());
        assert!(pointer_util::is_aligned_to(obj_1.unwrap().ptr, 16));
        let obj_2 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_2.is_some());
        assert!(pointer_util::is_aligned_to(obj_2.unwrap().ptr, 16));
        let obj_3 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_3.is_some());
        assert!(pointer_util::is_aligned_to(obj_3.unwrap().ptr, 16));
    }
}