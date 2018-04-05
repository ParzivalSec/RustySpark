use super::allocators::base::{ Allocator, MemoryBlock, BasicAllocator };
use super::bounds_checker::base::{ BoundsChecker };

///
/// A MemoryRealm is a combination of an allocation strategy and a bounds checking
/// strategy to combine each possible allocator with different bounds checking variations.
/// This system of a memory realm could be extended by implementing further memory tracking
/// and thread synchronisation strategies which would allow for an even broader variation of
/// memory realms.
///
pub struct BasicMemoryRealm<A: Allocator + BasicAllocator, B: BoundsChecker + Default> {
    allocator: A,
    bounds_checker: B,
}

impl<A: Allocator, B: BoundsChecker + Default> BasicMemoryRealm<A, B>
    where A: Allocator + BasicAllocator<AllocatorImplementation = A> {
    pub fn new(size: usize) -> BasicMemoryRealm<A, B> {
        BasicMemoryRealm {
            allocator: A::new(size),
            bounds_checker: Default::default(),
        }
    }

    pub fn alloc(&self, size: usize, alignment: usize) -> Option<MemoryBlock> {
        let canary_size = self.bounds_checker.get_canary_size() as usize;
        let total_allocation_size = size + (canary_size * 2) as usize;
        
        let block = self.allocator.alloc(total_allocation_size, alignment, canary_size);
        
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
            let allocated_ptr = mem_block.ptr.offset(-(canary_size as isize));
            let allocation_size = self.allocator.get_allocation_size(&mem_block);

            self.bounds_checker.validate_front_canary(allocated_ptr);
            self.bounds_checker.validate_back_canary(allocated_ptr.offset((allocation_size + canary_size) as isize));

            self.allocator.dealloc(mem_block);
        }
    }

    pub unsafe fn reset(&self) {
        self.allocator.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::allocators;
    use super::super::bounds_checker;
    use super::super::super::pointer_util;

    #[test]
    fn linear_alloc_simple_bounds_checking_realm() {
        type SimpleRealm = BasicMemoryRealm<allocators::linear_allocator::LinearAllocator, bounds_checker::simple_bounds_checker::SimpleBoundsChecker>;

        let realm: SimpleRealm = SimpleRealm::new(100);

        let ptr = realm.alloc(4, 1).unwrap().ptr;

        let front_marker = unsafe{ *(ptr.offset(-4) as *mut u32) };
        assert_eq!(front_marker, 0xCA);
        let back_marker = unsafe { *(ptr.offset(4) as *mut u32) };
        assert_eq!(back_marker, 0xCA);

    }
}

