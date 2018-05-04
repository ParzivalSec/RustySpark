use std::ptr;
use std::slice;
use std::mem;

use std::ops::{ Index, IndexMut, Deref, DerefMut };

use mem::virtual_mem;
use spark_core::{math_util, freelist::FreeList };

///
/// A Handle abstracts a pointer to an internal resource of the HandleMap
/// that is not affected by moving the resource it references in the 
/// internal memory. When handed back to the HandleMap, the user can
/// receive a reference to the object the handle refers to.
///
pub type Handle = usize;

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
pub struct HandleMap<'a, T: 'a> {
    dense_array:    &'a mut [T],
    handle_array:   &'a mut [HandleData],
    meta_array:     &'a mut [LookupMeta],
    freelist:       FreeList,
    size:           u32,
    max_size:       u32,
}

fn allocate_mem(size: usize) -> *mut u8 {
   let v_mem = virtual_mem::reserve_address_space(size).unwrap();
   let p_mem = virtual_mem::commit_physical_memory(v_mem, size).unwrap();
    p_mem
} 

impl<'a, T> HandleMap<'a, T> {
    pub fn new(max_size: u32) -> Self {
        // TOOD: To be comparable with the Spark++ equivalent of the HashMap that uses new[]/delete[] we
        // should also use the language's default allocator here and go with liballoc's alloc 
        let dense_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size as usize * mem::size_of::<T>(), virtual_mem::get_page_size()));
        let sparse_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size as usize * mem::size_of::<HandleData>(), virtual_mem::get_page_size()));
        let meta_arr_mem = allocate_mem(math_util::round_to_next_multiple(max_size as usize * mem::size_of::<LookupMeta>(), virtual_mem::get_page_size()));

        unsafe {
            for idx in 0 .. max_size {
                let uninit_handle_data = &mut *(sparse_arr_mem.offset((idx as usize * mem::size_of::<HandleData>()) as isize) as *mut HandleData);
                uninit_handle_data.sparse_array_idx = idx;
                uninit_handle_data.generation = 0;
            }
            
            HandleMap {
                dense_array:    slice::from_raw_parts_mut(dense_arr_mem as *mut T, max_size as usize),
                handle_array:   slice::from_raw_parts_mut(sparse_arr_mem as *mut HandleData, max_size as usize),
                meta_array:     slice::from_raw_parts_mut(meta_arr_mem as *mut LookupMeta, max_size as usize),
                freelist:       FreeList::new_from(
                                    sparse_arr_mem, 
                                    sparse_arr_mem.offset((max_size as usize * mem::size_of::<HandleData>()) as isize), 
                                    mem::size_of::<HandleData>()
                                ),
                size:           0,
                max_size,
            }
        }
    }

    pub fn insert(&mut self, item: T) -> Option<Handle> {
        {
            let enough_capacity_for_element = self.size < self.max_size;
            debug_assert!(enough_capacity_for_element, "Item count reached maximum, cannot insert anymore! Maybe alter the maximum size?");
        }

        if !self.freelist.empty() {
            unsafe {
                let handle_data = &mut *(self.freelist.get_block() as *mut HandleData);
                handle_data.dense_array_idx = self.size as usize;

                let internal_id = InternalHandleRepr { 
                    sparse_array_idx : handle_data.sparse_array_idx, 
                    generation: handle_data.generation
                };

                self.meta_array[self.size as usize].dense_to_sparse_idx = internal_id.sparse_array_idx;
                self.dense_array[self.size as usize] = item;

                self.size += 1;

                return Some(mem::transmute::<InternalHandleRepr, Handle>(internal_id))
            }
        }

        None
    }

    pub fn insert_copy(&mut self, item : &T) -> Option<Handle> 
        where T: Clone
    {
        self.insert(item.clone())
    }

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let mut removed_item = None;
        let internal_id = unsafe { mem::transmute::<Handle, InternalHandleRepr>(handle) };

        {
            let handle_index_in_range = internal_id.sparse_array_idx < self.size;
            debug_assert!(handle_index_in_range, "Index stored in the handle was out of range!");
        }

        let HandleData { dense_array_idx, sparse_array_idx: _sparse_array_idx, generation } = self.handle_array[internal_id.sparse_array_idx as usize];
        let is_generation_valid = internal_id.generation  == generation;
        if !is_generation_valid {
            return removed_item
        }

        let last_item_idx = self.size - 1;

        unsafe {
            removed_item = Some(ptr::read(&self.dense_array[dense_array_idx] as *const T));   
            self.dense_array.swap(dense_array_idx, last_item_idx as usize);
        }

        let moved_item_idx = self.meta_array[last_item_idx as usize].dense_to_sparse_idx;
        self.handle_array[moved_item_idx as usize].dense_array_idx = dense_array_idx;

        self.freelist.return_block((&mut self.handle_array[internal_id.sparse_array_idx as usize] as *mut HandleData) as *mut u8);

        (&mut self.handle_array[internal_id.sparse_array_idx as usize]).generation += 1;
        self.size = last_item_idx;

        removed_item
    }

    pub fn clear(&mut self) {
        for idx in 0..self.size as usize {
            let _ = self.dense_array[idx];
            self.handle_array[idx].generation += 1;
        }

        self.freelist = FreeList::new_from(
            (&mut self.handle_array[0] as *mut HandleData) as *mut u8,
            (&mut self.handle_array[self.size as usize - 1] as *mut HandleData) as *mut u8,
            mem::size_of::<HandleData>()
        );
    }

    pub fn at(&self, handle: Handle) -> &T {
        let internal_id = unsafe { mem::transmute::<Handle, InternalHandleRepr>(handle) };

        {
            let handle_index_in_range = internal_id.sparse_array_idx < self.size;
            debug_assert!(handle_index_in_range, "Index stored in the handle was out of range!");
        }

        let handle_data = &self.handle_array[internal_id.sparse_array_idx as usize];

        {
            let valid_generation = internal_id.generation == handle_data.generation;
            debug_assert!(valid_generation, "Generation of the handle was outdated or corrupted!");
        }

        &self.dense_array[handle_data.dense_array_idx as usize]
    }

    pub fn at_mut(&mut self, handle: Handle) -> &mut T {
        let internal_id = unsafe { mem::transmute::<Handle, InternalHandleRepr>(handle) };

        {
            let handle_index_in_range = internal_id.sparse_array_idx < self.size;
            debug_assert!(handle_index_in_range, "Index stored in the handle was out of range!");
        }

        let handle_data = &self.handle_array[internal_id.sparse_array_idx as usize];

        {
            let valid_generation = internal_id.generation == handle_data.generation;
            debug_assert!(valid_generation, "Generation of the handle was outdated or corrupted!");
        }

        &mut self.dense_array[handle_data.dense_array_idx as usize]
    }

    pub fn is_valid(&self, handle: Handle) -> bool {
        let internal_id = unsafe { mem::transmute::<Handle, InternalHandleRepr>(handle) };
    
        if internal_id.sparse_array_idx >= self.size {
            return false
        }

        let handle_data = &self.handle_array[internal_id.sparse_array_idx as usize];
        let generation_valid = handle_data.generation == internal_id.generation;
        let item_idx_valid = handle_data.dense_array_idx < self.size as usize;

        generation_valid && item_idx_valid
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn max_size(&self) -> u32 {
        self.max_size
    }
}

impl<'a, T> Index<Handle> for HandleMap<'a, T> {
    type Output = T;

    fn index(&self, index: Handle) -> &T {
        self.at(index)
    }
}

impl<'a, T> IndexMut<Handle> for HandleMap<'a, T> {
    fn index_mut(&mut self, index: Handle) -> &mut T {
        self.at_mut(index)
    }
}

impl<'a, T> Deref for HandleMap<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            ::std::slice::from_raw_parts(&self.dense_array[0], self.size as usize)
        }
    }
}

impl<'a, T> DerefMut for HandleMap<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            ::std::slice::from_raw_parts_mut(&mut self.dense_array[0], self.size as usize)
        }
    }
}

impl<'a, T> Drop for HandleMap<'a, T> {
    fn drop(&mut self) {
        if self.size != 0 {
            for idx in 0..self.size {
                let _ = self.dense_array[idx as usize];
            }

            virtual_mem::free_address_space((&mut self.dense_array[0] as *mut T) as *mut u8);
            virtual_mem::free_address_space((&mut self.handle_array[0] as *mut HandleData) as *mut u8);
            virtual_mem::free_address_space((&mut self.meta_array[0] as *mut LookupMeta) as *mut u8);
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
    fn construction() {
        let handle_map: HandleMap<Item> = HandleMap::new(100);

        assert_eq!(handle_map.max_size(), 100, "HandleMap max size was not 100");
        assert_eq!(handle_map.size(), 0, "HandleMap's initial size was not 0");
    }

    #[test]
    fn insert() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        let item_ref = handle_map.at(item_handle);
        assert_eq!(item_ref.data, 42, "Returned reference did not point to proper item");
    }

    #[test]
    fn insert_multiple() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle_0 = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        let item_handle_1 = handle_map.insert(Item { data: 43 }).unwrap();
        assert_eq!(handle_map.size(), 2, "Size was not updated after inserting a new item");
        
        let item_handle_2 = handle_map.insert(Item { data: 44 }).unwrap();
        assert_eq!(handle_map.size(), 3, "Size was not updated after inserting a new item");

        let item_ref_0 = handle_map.at(item_handle_0);
        assert_eq!(item_ref_0.data, 42, "Returned reference did not point to proper item");
        let item_ref_1 = handle_map.at(item_handle_1);
        assert_eq!(item_ref_1.data, 43, "Returned reference did not point to proper item");
        let item_ref_2 = handle_map.at(item_handle_2);
        assert_eq!(item_ref_2.data, 44, "Returned reference did not point to proper item");
    }

    #[test]
    fn at_mut() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        let item_ref = handle_map.at_mut(item_handle);
        assert_eq!(item_ref.data, 42, "Returned mutable reference did not point to proper item");
        item_ref.data = 66;
        assert_eq!(item_ref.data, 66, "Mutating the returned reference did not alter the value");
    }

    #[test]
    fn index() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        let item_ref = &handle_map[item_handle];
        assert_eq!(item_ref.data, 42, "Returned mutable reference did not point to proper item");
    }

    #[test]
    fn mut_index() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        let item_ref = &mut handle_map[item_handle];
        assert_eq!(item_ref.data, 42, "Returned mutable reference did not point to proper item");
        item_ref.data = 66;
        assert_eq!(item_ref.data, 66, "Returned mutable reference was not changed when assigning new value");
    }

    #[test]
    fn remove() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
        assert_eq!(handle_map.size(), 1, "Size was not updated after inserting a new item");

        handle_map.remove(item_handle);
        assert!(!handle_map.is_valid(item_handle), "Handle was still valid after remove()");
    }

    #[test]
    #[should_panic(expected = "Item count reached maximum, cannot insert anymore! Maybe alter the maximum size?")]
    fn assert_on_max_size() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        for idx in 0..100 {
            let _ = handle_map.insert(Item { data: 42 }).unwrap();
            assert_eq!(handle_map.size(), idx + 1, "Size was not updated after inserting a new item");
        }

        let _item_handle = handle_map.insert(Item { data: 42 }).unwrap();
    }

    #[test]
    fn clear() {
        let mut handle_storage: Vec<Handle> = Vec::with_capacity(100);
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        for _ in 0..100 {
            let item_handle = handle_map.insert(Item { data: 42 }).unwrap();
            handle_storage.push(item_handle);
        }

        handle_map.clear();

        for idx in 0..100 {
            assert!(!handle_map.is_valid(handle_storage[idx]), "Handle was still valid after clear()");
        }
    }

    #[test]
    fn iterate_indexed() {
        let mut handle_map: HandleMap<Item> = HandleMap::new(100);

        for idx in 0..100 {
            let _ = handle_map.insert(Item { data: idx }).unwrap();
        }

        for idx in 0..100 {
            assert_eq!(handle_map[idx].data, idx, "HandleMap [index] did not return a proper element of the dense array");

            if idx < 99 {
                assert_eq!(&handle_map[idx + 1] as *const Item, unsafe { (&mut handle_map[idx] as *mut Item).offset(1) }, "Items were not contigous in memory");
            }
        }
    }
}