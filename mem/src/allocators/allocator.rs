use std::marker::PhantomData;

///
/// Zero-cost abstraction over an allocation done by an allocator
///
#[derive(Debug)]
pub struct AllocatorMem<'a> {
    pub ptr: *mut u8,
    pub _phantom_slice: PhantomData<&'a mut [u8]>,
}

impl<'a> AllocatorMem<'a> {
    
    pub fn new(ptr: *mut u8) -> Self {
        debug_assert!(!ptr.is_null());
        AllocatorMem {
            ptr,
            _phantom_slice: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.ptr.is_null() }
}

///
/// Base trait that indicates that a type is able to fullfil allocation requests
/// issued by the user
///
pub trait Allocator {
    fn alloc(&self, size: usize, alignment: usize, offset: usize) -> Option<AllocatorMem>;
    fn dealloc(&self, memory: AllocatorMem);
    fn reset(&self);
    fn get_allocation_size(&self, memory: &AllocatorMem) -> usize;
}

///
/// Marker trait to implicate that an Allocator can grow
///
pub trait GrowingAllocator {}
