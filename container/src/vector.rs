use std::mem;
use std::ptr::{ Unique, self };
use std::option::{ Option };

use spark_core::math_util;
use mem::virtual_mem;

const INITIAL_GROW_AMOUNT: usize = 8; // Amount of element the vector grows the first time on push when it was empty
const MAX_VECTOR_CAPACITY: usize = 1024 * 1024 * 1024; // One vector can hold a max of 1GB at a time

pub struct Vector<T> {
    virtual_mem_begin:  *mut u8,
    virtual_mem_end:    *mut u8,
    internal_array_begin: Unique<T>,
    internal_array_end: *mut u8,
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
            virtual_mem_begin:      vector_virtual_mem,
            virtual_mem_end:        unsafe { vector_virtual_mem.offset(MAX_VECTOR_CAPACITY as isize) },
            internal_array_begin:   unsafe { Unique::new_unchecked(vector_virtual_mem as *mut T) },
            internal_array_end:     vector_virtual_mem,
            capacity:               0,
            size:                   0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.size == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::write(self.internal_array_begin.as_ptr().offset(self.size as isize), item);
        }

        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        }
        else {
            self.size -= 1;
            unsafe {
                Some(ptr::read(self.internal_array_begin.as_ptr().offset(self.size as isize)))
            }
        }
    }

    pub fn resize(&mut self, new_size: usize) 
        where T: Default
    {
        unimplemented!();
    }

    pub fn resize_with_template(&mut self, new_size: usize, object: &T)
        where T: Clone 
    {
        unimplemented!();
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        unimplemented!();
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

    fn grow(&mut self) {
        let elements_to_grow = if self.capacity != 0 {
           self.capacity * 2
        }
        else {
            INITIAL_GROW_AMOUNT
        };

        {
            let virtual_address_space_exhausted = self.internal_array_begin.as_ptr() as *mut u8 == self.virtual_mem_end;
            debug_assert!(!virtual_address_space_exhausted, "Not enough address space to grow further");
        }

        let page_bytes_to_grow = math_util::round_to_next_multiple(elements_to_grow * mem::size_of::<T>(), virtual_mem::get_page_size());

        let is_enough_space_for_requested_pages = unsafe { self.internal_array_end.offset(page_bytes_to_grow as isize) <= self.virtual_mem_end };
        let bytes = if is_enough_space_for_requested_pages {
            page_bytes_to_grow
        }
        else {
            let remaining_virtual_address_space = self.virtual_mem_end as usize - self.internal_array_end as usize;
            math_util::round_to_previous_multiple(remaining_virtual_address_space, virtual_mem::get_page_size())
        };

        let ptr = match { virtual_mem::commit_physical_memory(self.internal_array_end, bytes) } {
            None => ptr::null_mut(),
            Some(mem) => mem,
        };

        if ptr.is_null() {
            debug_assert!(true, "Vector run out of memory due to an unknow error");
        }

        self.internal_array_end = unsafe { ptr.offset(bytes as isize) };
        self.capacity = self.capacity + (bytes / mem::size_of::<T>());
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        if self.capacity == 0 {
            while let Some(_) = self.pop() {}
            virtual_mem::free_address_space(self.internal_array_begin.as_ptr() as *mut u8);
        }
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

    #[test]
    fn push_back_data() {
        let mut vec: Vector<Item> = Vector::new();

        vec.push(Item { data: 0xCC });
        vec.push(Item { data: 0xCC });
    }
}