use super::allocators::base::{ Allocator, MemoryBlock, TypedAllocator };
use super::bounds_checker::base::{ BoundsChecker };

///
/// A TypedMemoryRealm is a combination of an allocation strategy that assumes every allocation
/// is (at most - it's possible to vary inside of one block) from the same size and a bounds checking
/// strategy to combine each possible allocator with different bounds checking variations.
/// This system of a memory realm could be extended by implementing further memory tracking
/// and thread synchronisation strategies which would allow for an even broader variation of
/// memory realms.
///
pub struct TypedMemoryRealm<A: Allocator + TypedAllocator, B: BoundsChecker + Default> {
    allocator: A,
    bounds_checker: B,
}

impl<A: Allocator + TypedAllocator, B: BoundsChecker + Default> TypedMemoryRealm<A, B>
    where A: Allocator + TypedAllocator<AllocatorImplementation = A> {
    pub fn new(element_size: usize, element_count: usize, element_alignment: usize) -> TypedMemoryRealm<A, B> {
        let bounds_checker: B = Default::default();
        
        // Here we alter the element_size by twice the canary size to
        // ensure that a mem block to hold one instance of element
        // is big enough to also store the two canary values if provided
        let canary_size = bounds_checker.get_canary_size() as usize;
        let type_size_with_offset = element_size + (canary_size * 2);

        TypedMemoryRealm {
            allocator: A::new(type_size_with_offset, element_count, element_alignment, canary_size),
            bounds_checker,
        }
    }

    pub fn alloc(&self, size: usize, alignment: usize) -> Option<MemoryBlock> {
        let canary_size = self.bounds_checker.get_canary_size() as usize;
        let _offset_not_needed = 0;
        
        let block = self.allocator.alloc_raw(size, alignment, _offset_not_needed);

        if block.is_none() {
            return None;
        }
        
        let user_ptr = block.unwrap().ptr;
        
        unsafe {
            self.bounds_checker.write_canary(user_ptr);
            self.bounds_checker.write_canary(user_ptr.offset((size + canary_size) as isize));

            Some(MemoryBlock::new(user_ptr.offset(canary_size as isize)))
        }
    }

    pub fn dealloc(&self, mem_block: MemoryBlock) {
        let canary_size = self.bounds_checker.get_canary_size() as usize;

        unsafe {
            let original_mem_block = MemoryBlock { ptr: mem_block.ptr.offset(-(canary_size as isize)), ..mem_block };
            let allocation_size = self.allocator.get_allocation_size(&original_mem_block);

            self.bounds_checker.validate_front_canary(original_mem_block.ptr);
            self.bounds_checker.validate_back_canary(original_mem_block.ptr.offset((allocation_size + canary_size) as isize));

            self.allocator.dealloc_raw(original_mem_block);
        }
    }

    pub unsafe fn reset(&self) {
        self.allocator.reset();
    }
}

#[cfg(test)]
mod tests {
    use std;
    use spark_core::pointer_util;

    use super::*;
    use super::super::allocators;
    use super::super::bounds_checker;

    struct Particle {
        pub lifetime: f32,
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    #[test]
    fn typed_realm_with_pool_alloc_and_bounds_checking() {
        type TypedPool = TypedMemoryRealm<allocators::pool_allocator::PoolAllocator, bounds_checker::simple_bounds_checker::SimpleBoundsChecker>;
    
        let typed_pool = TypedPool::new(std::mem::size_of::<Particle>(), 10, 4);
        let mut particles: Vec<MemoryBlock> = Vec::new();

        for i in 0 .. 10 {
            let mem = typed_pool.alloc(std::mem::size_of::<Particle>(), 4);
            assert!(mem.is_some(), "Allocator mem block was none!");
            let mem_block = mem.unwrap();
            assert!(pointer_util::is_aligned_to(mem_block.ptr, 4), "Allocated block was not properly aligned to 4 byte-boundary");
            let particle = unsafe { &mut *(mem_block.ptr as *mut Particle) };

            particle.lifetime = 1.0;
            particle.r = i as u8;
            particle.g = i as u8;
            particle.b = i as u8;

            particles.push(mem_block);
        }

        for i in 0 .. 10 {
            let mem_block = &particles[i];
            let particle = unsafe { &mut *(mem_block.ptr as *mut Particle) };

            assert_eq!(particle.lifetime, 1.0);
            assert_eq!(particle.r, i as u8);
            assert_eq!(particle.g, i as u8);
            assert_eq!(particle.b, i as u8);
        }

        for particle_mem in particles {
            typed_pool.dealloc(particle_mem);
        }
    }
}