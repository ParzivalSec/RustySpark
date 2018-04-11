use std::mem;
use std::ptr::{ Unique, self };
use std::option::{ Option };
use std::ops::{ Deref, DerefMut };

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
            let grow_in_bytes = self.get_grow_size() * mem::size_of::<T>();
            self.grow(grow_in_bytes);
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

    pub fn erase(&mut self, index: usize) 
    {
        {
            let index_in_range = index < self.size;
            debug_assert!(index_in_range, "Index was out of range");
        }

        let _erased = unsafe { ptr::read(self.internal_array_begin.as_ptr().offset(index as isize)) };

        self.size -= 1;

        unsafe {
            ptr::copy(
                self.internal_array_begin.as_ptr().offset(index as isize + 1),
                self.internal_array_begin.as_ptr().offset(index as isize),
                self.size - index,
            );
        }
    }

    pub fn erase_range(&mut self, begin: usize, end: usize)
    {
        if begin == end { return; }

        {
            let begin_bigger_than_end = begin <= end;
            debug_assert!(begin_bigger_than_end, "End index was out of range");
            let end_in_range = end < self.size;
            debug_assert!(end_in_range, "Endindex was out of range");
        }

        let erasing_element_count = (end - begin) + 1;

        for idx in begin..erasing_element_count + 1 {
            let _ = unsafe { ptr::read(self.internal_array_begin.as_ptr().offset(idx as isize)) };
        }

        self.size -= erasing_element_count;

        unsafe {
            ptr::copy(
                self.internal_array_begin.as_ptr().offset(end as isize + 1),
                self.internal_array_begin.as_ptr().offset(begin as isize),
                self.size.checked_sub(begin).unwrap(),
            );
        }
    }

    pub fn resize(&mut self, new_size: usize) 
        where T: Default
    {
        	{
				let resize_request_exceeds_available_range = new_size > self.max_elements();
				debug_assert!(!resize_request_exceeds_available_range, "Resize requested more elements then the max capacity possible");
			}

			if new_size == self.size { return; }

            if new_size > self.size {
                if new_size > self.capacity {
                    let grow_in_bytes = (new_size - self.capacity) * mem::size_of::<T>();
                    self.grow(grow_in_bytes);
                }

                for idx in self.size..new_size {
                    let new_item: T = Default::default();
                    unsafe { ptr::write(self.internal_array_begin.as_ptr().offset(idx as isize), new_item) };
                }
            }
            else {
                for _ in self.size..new_size {
                    let _ = self.pop();
                }
            }

            self.size = new_size;
    }

    pub fn resize_with_template(&mut self, new_size: usize, object: &T)
        where T: Clone 
    {
       	{
				let resize_request_exceeds_available_range = new_size > self.max_elements();
				debug_assert!(!resize_request_exceeds_available_range, "Resize requested more elements then the max capacity possible");
			}

			if new_size == self.size { return; }

            if new_size > self.size {
                if new_size > self.capacity {
                    let grow_in_bytes = (new_size - self.capacity) * mem::size_of::<T>();
                    self.grow(grow_in_bytes);
                }

                for idx in self.size..new_size {
                    unsafe { ptr::write(self.internal_array_begin.as_ptr().offset(idx as isize), object.clone()) };
                }
            }
            else {
                for _ in self.size..new_size {
                    let _ = self.pop();
                }
            }

            self.size = new_size;
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        {
            let enough_maximum_capacity = new_capacity <= self.max_elements();
            debug_assert!(enough_maximum_capacity, "Requested capacity exceeds total available capacity for this vector");    
        }

        if new_capacity <= self.capacity {
            return;
        }

        let new_capacity_in_bytes = (new_capacity - self.capacity) * mem::size_of::<T>();
        self.grow(new_capacity_in_bytes);
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

    fn max_elements(&self) -> usize {
        MAX_VECTOR_CAPACITY / mem::size_of::<T>()
    }

    fn get_grow_size(&self) -> usize {
        if self.capacity != 0 {
            self.capacity() * 2
        }
        else {
            INITIAL_GROW_AMOUNT
        }
    }

    fn grow(&mut self, bytes: usize) {
        {
            let virtual_address_space_exhausted = self.internal_array_begin.as_ptr() as *mut u8 == self.virtual_mem_end;
            debug_assert!(!virtual_address_space_exhausted, "Not enough address space to grow further");
        }

        let page_bytes_to_grow = math_util::round_to_next_multiple(bytes, virtual_mem::get_page_size());

        let is_enough_space_for_requested_pages = unsafe { self.internal_array_end.offset(page_bytes_to_grow as isize) <= self.virtual_mem_end };
        let grow_by_bytes = if is_enough_space_for_requested_pages {
            page_bytes_to_grow
        }
        else {
            let remaining_virtual_address_space = self.virtual_mem_end as usize - self.internal_array_end as usize;
            math_util::round_to_previous_multiple(remaining_virtual_address_space, virtual_mem::get_page_size())
        };

        let ptr = match { virtual_mem::commit_physical_memory(self.internal_array_end, grow_by_bytes) } {
            None => ptr::null_mut(),
            Some(mem) => mem,
        };

        if ptr.is_null() {
            debug_assert!(true, "Vector run out of memory due to an unknow error");
        }

        self.internal_array_end = unsafe { ptr.offset(grow_by_bytes as isize) };
        self.capacity = self.capacity + (grow_by_bytes / mem::size_of::<T>());
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            ::std::slice::from_raw_parts(self.internal_array_begin.as_ptr(), self.size)
        }
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            ::std::slice::from_raw_parts_mut(self.internal_array_begin.as_ptr(), self.size)
        }
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

    impl Default for Item {
        fn default() -> Item {
            Item {
                data: 42,
            }
        }
    }

    impl Drop for Item {
        fn drop(&mut self) {
            println!("Item dropped");
        }
    }

    #[test]
    fn create_new_vector_empty() {
        let vec: Vector<Item> = Vector::new();

        assert!(vec.size == 0, "Vector was initialized with non zero size");
        assert!(vec.capacity == 0, "Vector was initialized with non zero capacity");
    }

    #[test]
    fn push_data() {
        let mut vec: Vector<Item> = Vector::new();

        vec.push(Item { data: 0xCC });
        vec.push(Item { data: 0xDD });

        assert_eq!(vec[0].data, 0xCC);
        assert_eq!(vec[1].data, 0xDD);

        assert_eq!(vec.size(), 2);
        assert_eq!(vec.capacity(), 512);
    }

    #[test]
    fn pop_data() {
        let mut vec: Vector<Item> = Vector::new();

        vec.push(Item { data: 0xCC });
        vec.push(Item { data: 0xDD });

        assert_eq!(vec.pop().unwrap().data, 0xDD);
        assert_eq!(vec.pop().unwrap().data, 0xCC);

        assert_eq!(vec.size(), 0);
        assert_eq!(vec.capacity(), 512);
    }

    #[test]
    fn erase_data() {
        let mut vec: Vector<Item> = Vector::new();

        vec.push(Item { data: 0xCC });
        vec.push(Item { data: 0xDD });
        vec.push(Item { data: 0xEE });
        vec.push(Item { data: 0xFF });

        vec.erase(1);

        assert_eq!(vec.size(), 3);
        assert_eq!(vec.capacity(), 512);

        assert_eq!(vec[0].data, 0xCC);
        assert_eq!(vec[1].data, 0xEE);
        assert_eq!(vec[2].data, 0xFF);
    }

    #[test]
    fn erase_data_range() {
        let mut vec: Vector<Item> = Vector::new();

        vec.push(Item { data: 0xCC });
        vec.push(Item { data: 0xDD });
        vec.push(Item { data: 0xEE });
        vec.push(Item { data: 0xFF });

        vec.erase_range(1, 2);

        assert_eq!(vec.size(), 2);
        assert_eq!(vec.capacity(), 512);

        assert_eq!(vec[0].data, 0xCC);
        assert_eq!(vec[1].data, 0xFF);
    }

    #[test]
    fn reserve() {
        let mut vec: Vector<Item> = Vector::new();
    
        vec.reserve(600);

        assert_eq!(vec.size(), 0);
        assert_eq!(vec.capacity(), 1024);
    }

    #[test]
    fn resize_default() {
        let mut vec: Vector<Item> = Vector::new();
    
        vec.resize(4);

        assert_eq!(vec.size(), 4);
        assert_eq!(vec.capacity(), 512);

        assert_eq!(vec[0].data, 42);
        assert_eq!(vec[1].data, 42);
        assert_eq!(vec[2].data, 42);
        assert_eq!(vec[3].data, 42);
    }
    
}