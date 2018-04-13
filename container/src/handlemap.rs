use std::ptr::{ Unique, self };
use std::mem;

use mem::virtual_mem;
use spark_core::math_util;

///
/// A Handle abstracts a pointer to an internal resource of the HandleMap
/// that is not affected by moving the resource it references in the 
/// internal memory. When handed back to the HandleMap, the user can
/// receive a reference to the object the handle refers to.
///
pub struct Handle {
    pub handle_id: usize,
}

struct HandleData
{
    pub dense_array_idx:    usize,
    pub sparse_array_idx:   u32,
    pub generation:         u32,
}

struct LookupMeta {
    pub dense_to_sparse_idx: u32,
}

struct InternalHandleRepr {
    pub sparse_array_idx:   u32,
    pub generation:         u32,
}

///
///
///
struct HandleMap<T> {
    dense_array:    Unique<T>,
    handle_array:   Unique<HandleData>,
    meta_array:     Unique<LookupMeta>,
    size:           usize,
    max_size:       usize,
}

fn allocate_mem(size: usize) -> *mut u8 {
   let v_mem = virtual_mem::reserve_address_space(size).unwrap();
   let p_mem = virtual_mem::commit_physical_memory(v_mem, size).unwrap();
    p_mem
} 

impl<T> HandleMap<T> {
    pub fn new(max_size: usize) -> Self {
        let dense_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size * mem::size_of::<T>(), virtual_mem::get_page_size()));
        let sparse_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size * mem::size_of::<HandleData>(), virtual_mem::get_page_size()));
        let meta_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size * mem::size_of::<LookupMeta>(), virtual_mem::get_page_size()));
        
        HandleMap {
            dense_array: Unique::new(dense_arr_mem as *mut T).unwrap(),
            handle_array: Unique::new(sparse_arr_mem as *mut HandleData).unwrap(),
            meta_array: Unique::new(meta_arr_mem as *mut LookupMeta).unwrap(),
            size: 0,
            max_size, 
        }
    }

    pub fn insert(&mut self, item: T) -> Handle {
        unimplemented!()
    }

    pub fn insert_copy(&mut self, item : &T) -> Handle 
        where T: Clone
    {
        unimplemented!()
    }

    pub fn remove(&mut self, handle: Handle) -> T {
        unimplemented!()
    }

    pub fn clear(&mut self) {
        unimplemented!()
    }

    pub fn at(&self) -> &T {
        unimplemented!()
    }

    pub fn at_mut(&self) -> &mut T {
        unimplemented!()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn is_valid(&self, handle: Handle) -> bool {
        unimplemented!()
    }
}