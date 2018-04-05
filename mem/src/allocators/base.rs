use std::marker::PhantomData;
use std::ptr::Unique;

///
/// Zero-cost abstraction over an allocation done by an allocator
///
#[derive(Debug)]
pub struct MemoryBlock<'a> {
    pub ptr: *mut u8,
    pub _phantom_slice: PhantomData<&'a mut [u8]>,
}

impl<'a> MemoryBlock<'a> {
    
    pub fn new(ptr: *mut u8) -> Self {
        debug_assert!(!ptr.is_null());
        MemoryBlock {
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
    fn alloc(&self, size: usize, alignment: usize, offset: usize) -> Option<MemoryBlock>;
    fn dealloc(&self, memory: MemoryBlock);
    fn reset(&self);
    fn get_allocation_size(&self, memory: &MemoryBlock) -> usize;
}

pub trait BasicAllocator {
    type AllocatorImplementation;
    fn new(size: usize) -> Self::AllocatorImplementation;
}

pub trait TypedAllocator {
    type AllocatorImplementation;
    fn new(element_size: usize, element_count: usize, element_alignment: usize, offset: usize) -> Self::AllocatorImplementation;
}