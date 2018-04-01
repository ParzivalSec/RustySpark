use std::marker::PhantomData;

///
/// Zero-cost abstraction over an allocation done by an allocator
///
#[derive(Debug)]
pub struct AllocatorMem<'a> {
    pub ptr: *mut u8,
    pub _phantom_slice: PhantomData<&'a mut [u8]>,
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
