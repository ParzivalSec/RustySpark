use std::mem;
use std::ptr::{ Unique, self };
use std::option::{ Option };
use mem::virtual_mem;

const MAX_VECTOR_CAPACITY: usize = 1024 * 1024 * 1024; // One vector can hold a max of 1GB at a time

pub struct Vector<T> {
    virtual_mem_begin:  *mut u8,
    virtual_mem_end:    *mut u8,
    physical_mem_begin: Unique<T>,
    physical_mem_end: *mut u8,
    capacity: usize,
    size: usize,
}

impl<T> Vector<T> {
    pub fn new() -> Self {
        debug_assert!(mem::size_of::<T>() != 0, "Vector cannot handel zero-sized types");
        
        let vector_virtual_mem = match { virtual_mem::reserve_address_space(MAX_VECTOR_CAPACITY) } {
            None => ptr::null_mut(),
            Some(ptr) => ptr,
        };

        debug_assert!(vector_virtual_mem != ptr::null_mut(), "Could not allocate any virtual memory for the vector");
        
        Vector {
            virtual_mem_begin:  vector_virtual_mem,
            virtual_mem_end:    vector_virtual_mem,
            physical_mem_begin: unsafe { Unique::new_unchecked(vector_virtual_mem as *mut T) },
            physical_mem_end:   vector_virtual_mem,
            capacity:           0,
            size:               0,
        }
    }

    pub fn resize(&mut self, new_size: usize) 
        where T: Default
    {
        unimplemented!();
    }

    pub fn resize(&mut self, new_size: usize, object: &T)
        where T: Clone 
    {
        unimplemented!();
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        unimplemented!();
    }

    pub fn push(&mut self, item: T) {
        unimplemented!();
    }

    pub fn pop(&mut self) -> Option<T> {
        None
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Item {
        pub data: usize,
    }

    #[test]
    fn create_new_vector_empty() {
        let vec: Vector<Item> = Vector::new();

        assert!(vec.size == 0, "Vector was initialized with non zero size");
        assert!(vec.capacity == 0, "Vector was initialized with non zero capacity");
    }
}