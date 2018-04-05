use std;
use std::marker::PhantomData;
use std::cell::RefCell;

use super::super::{ virtual_mem, pointer_util, freelist };
use super::base::{ Allocator, MemoryBlock, TypedAllocator };

///
/// The AllocationHeader struct describes meta-data
/// the allocator needs to store alongside of the 
/// allocations.
///
struct AllocationHeader {
    pub allocation_size: u32,
}

const ALLOCATION_META_SIZE: usize = std::mem::size_of::<AllocationHeader>();

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
            let allocation_meta_offset = (offset + ALLOCATION_META_SIZE) as isize;
            let aligned_ptr  = pointer_util::align_top(physical_address_space.offset(allocation_meta_offset), max_element_alignment) as *mut u8;
            let before_aligned_ptr = aligned_ptr.offset(-allocation_meta_offset);

            before_aligned_ptr
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

impl TypedAllocator for PoolAllocator {
    type AllocatorImplementation = PoolAllocator;

    fn new(max_element_size: usize, element_count: usize, max_element_alignment: usize, offset: usize) -> Self::AllocatorImplementation {
        let block_min_size = calculate_minimal_block_size(max_element_size + ALLOCATION_META_SIZE, max_element_alignment);
        let required_memory_size = (element_count * block_min_size) + max_element_alignment;

        PoolAllocator {
            storage: RefCell::new(PoolAllocatorStorage::new(
                required_memory_size,
                block_min_size,
                max_element_size,
                max_element_alignment,
                offset)
            ),
        }
    }
}

impl Allocator for PoolAllocator {    
    fn alloc(&self, size: usize, alignment: usize, _offset: usize) -> Option<MemoryBlock> {
        let storage = self.storage.borrow_mut();

        {
            let size_lesser_or_equal_max_element_size = size <= storage.max_element_size;
            debug_assert!(size_lesser_or_equal_max_element_size, "Alloc size has to be less or equal max element size");
            let alignment_lesser_or_equal_max_element_alignment = alignment <= storage.max_element_alignment;
            debug_assert!(alignment_lesser_or_equal_max_element_alignment, "Alloc alignment has to be less or equal max element alignment");
        }
        
        let mut ptr = storage.free_list.get_block();
        
        if ptr.is_null() {
            return None;
        }

        unsafe {
            let allocation_header = &mut *(ptr as *mut AllocationHeader);
            allocation_header.allocation_size = size as u32;
            ptr = ptr.offset(ALLOCATION_META_SIZE as isize);
        }

        Some(MemoryBlock {
            ptr,
            _phantom_slice: PhantomData,
        })
    }

    fn dealloc(&self, memory: MemoryBlock) {
        {
            // TODO: Asserts
        }

        let storage = self.storage.borrow_mut();
        let original_ptr = unsafe { memory.ptr.offset(-(ALLOCATION_META_SIZE as isize)) };
        storage.free_list.return_block(original_ptr);
    }

    fn reset(&self) {
        let mut storage = self.storage.borrow_mut();
        storage.free_list = freelist::FreeList::new_from(
            storage.first_block_ptr,
            storage.mem_end,
            storage.min_block_size
        );
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
    fn single_allocation_aligned_with_offset() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>() + 8, // Adding 8, for 4 byte offset at each end of block
            10,
            32,
            4
        );

        let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>() + 8, 32, 4);
        assert!(obj_0.is_some());
        let mem_block = obj_0.unwrap();
        assert!(!pointer_util::is_aligned_to(mem_block.ptr, 32));
        let offsetted_ptr = unsafe { mem_block.ptr.offset(4) };
        assert!(pointer_util::is_aligned_to(offsetted_ptr, 32));
    }

    #[test]
    fn multiple_allocations() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            1,
            0
        );

        for _ in 0 .. 3 {
            let obj = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0);
            assert!(obj.is_some());
        }
    }

    #[test]
    fn multiple_allocations_aligned() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            16,
            0
        );

        for _ in 0 .. 3 {
            let obj = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
            assert!(obj.is_some());
            assert!(pointer_util::is_aligned_to(obj.unwrap().ptr, 16));
        }
    }

    #[test]
    fn return_none_on_oom() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            16,
            0
        );

        // We can allocate more than 10 Particles bc of page size rounging when allocating virt-mem
        // In fact we are leaking memory inside of this loop bc an MemoryBlock is in the responsibility
        // of the user - and bc each get dropped at the end of the scope we leak the mem in the allocator
        // hence triggering the oom in the last allocation request (a later implemented AllocatorBox will
        // add a safety layer for mem-leaks, deallocating the MemoryBlock when dropped)
        for _ in 0 .. 11 {
            let obj_0 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
            assert!(obj_0.is_some());
        }

        let obj_1 = pool_alloc.alloc(std::mem::size_of::<Particle>(), 16, 0);
        assert!(obj_1.is_none());
    }

    #[test]
    fn allocation_do_not_invalidate_prev_ones() {
        let pool_alloc = PoolAllocator::new(
            std::mem::size_of::<Particle>(),
            10,
            16,
            0
        );
        
        let mut part_vec_0 = Vec::new();

        // Get 5 particles and fill them with value, remeber the blocks in a vec
        for i in 0 .. 5 {
            let part_mem = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0).unwrap();
            let particle: &mut Particle = unsafe { &mut *(part_mem.ptr as *mut Particle) };

            particle.lifetime = i as f32;
            particle.speed = i;

            part_vec_0.push(part_mem);
        }

        let mut part_vec_1 = Vec::new();
        // Get another 5 particles into another vec
        for i in 5 .. 10 {
            let part_mem = pool_alloc.alloc(std::mem::size_of::<Particle>(), 1, 0).unwrap();
            let particle: &mut Particle = unsafe { &mut *(part_mem.ptr as *mut Particle) };

            particle.lifetime = i as f32;
            particle.speed = i;

            part_vec_1.push(part_mem);
        }

        for idx in 0 .. 5 {
            let vec_0_part: &mut Particle = unsafe { &mut *(part_vec_0[idx].ptr as *mut Particle) };
            let vec_1_part: &mut Particle = unsafe { &mut *(part_vec_1[idx].ptr as *mut Particle) };

            assert!(vec_0_part.lifetime == idx as f32, "Particle lifetime from vec 0 was corrupted");
            assert!(vec_0_part.speed == idx, "Particle speed from vec 0 was corrupted");
            assert!(vec_1_part.lifetime == (idx + 5) as f32, "Particle lifetime from vec 1 was corrupted");
            assert!(vec_1_part.speed == idx + 5, "Particle speed from vec 1 was corrupted");
        }
    }
}