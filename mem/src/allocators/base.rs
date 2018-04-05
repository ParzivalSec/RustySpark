use std::marker::PhantomData;
use std::{ mem, ptr, intrinsics, ptr::Unique, ops::Deref, ops::DerefMut };

///
/// Zero-cost abstraction over an allocation done by an allocator
///
#[derive(Debug)]
pub struct MemoryBlock<'a> {
    pub ptr: *mut u8,
    pub _marker: PhantomData<&'a [u8]>,
}

impl<'a> MemoryBlock<'a> {
    
    pub fn new(ptr: *mut u8) -> Self {
        debug_assert!(!ptr.is_null());
        MemoryBlock {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn empty() -> Self {
        MemoryBlock {
            ptr: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.ptr.is_null() }
}

pub struct AllocatorBox<'a, T: 'a + ?Sized, A: 'a + Allocator + ?Sized> {
    instance: Unique<T>,
    allocator: &'a A,
}

impl<'a, T: ?Sized, A: Allocator + ?Sized> AllocatorBox<'a, T, A> {
    pub fn instance_from(self) -> T where T: Sized {
        let instance = unsafe { ptr::read(self.instance.as_ptr()) };
        let mem_block = MemoryBlock::new(self.instance.as_ptr() as *mut u8);
        self.allocator.dealloc_raw(mem_block);
        mem::forget(self);
        instance
    }

    pub unsafe fn as_memory_block(&self) -> MemoryBlock {
        MemoryBlock::new(self.instance.as_ptr() as *mut u8)
    }
}

impl<'a, T: ?Sized, A: Allocator + ?Sized> Deref for AllocatorBox<'a, T, A> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { self.instance.as_ref() }
    }
}

impl<'a, T: ?Sized, A: Allocator + ?Sized> DerefMut for AllocatorBox<'a, T, A> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.instance.as_mut() }
    }
}

impl<'a, T: ?Sized, A: Allocator + ?Sized> Drop for AllocatorBox<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            intrinsics::drop_in_place(self.instance.as_ptr());
            self.allocator.dealloc_raw(MemoryBlock::new(self.instance.as_ptr() as *mut u8));
        }
    }
}

///
/// Base trait that indicates that a type is able to fullfil allocation requests
/// issued by the user
///
pub trait Allocator {
    fn alloc<T>(&self, value: T, alignment: usize, offset: usize) -> Option<AllocatorBox<T, Self>> 
    where Self: Sized,
    {
        match { self.alloc_raw(mem::size_of::<T>(), alignment, offset) } {
            Some(block) => {
                unsafe { ptr::write(block.ptr as *mut T, value); }

                Some(AllocatorBox {
                    instance: Unique::new(block.ptr as *mut T).expect("Could not create AllocatorBox from valid MemoryBlock"),
                    allocator: self,
                })
            },
            None => None,
        }
    }

    fn alloc_raw(&self, size: usize, alignment: usize, offset: usize) -> Option<MemoryBlock>;
    fn dealloc_raw(&self, memory: MemoryBlock);
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