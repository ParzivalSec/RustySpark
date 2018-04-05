use std::{ mem, cell::Cell };

pub struct FreeList {
    pub list: Cell<*mut u8>,
}

impl FreeList {
    pub fn new_from(begin: *mut u8, end: *mut u8, block_size: usize) -> FreeList {
        
        {
            let block_greater_or_equal_pointer_size = block_size >= mem::size_of::<*mut u8>();
            debug_assert!(block_greater_or_equal_pointer_size, "Block size needs to be greater or equal to a pointer size");
        }

        let mem_range_in_bytes = end as usize - begin as usize;
        let number_of_blocks = mem_range_in_bytes / block_size;
        let signed_block_size = block_size as isize;
        let free_list: *mut u8 = begin;

        let mut current: *mut *mut u8 = free_list as *mut *mut u8;
        let mut memory: *mut u8 = begin;
        memory = unsafe { memory.offset(signed_block_size) };
        
        unsafe {
            for _ in 0 .. number_of_blocks {         
                *current = memory.offset(signed_block_size);
                current = *current as *mut *mut u8;
                memory = memory.offset(signed_block_size);
            }
        }

        FreeList {
            list: Cell::new(free_list),
        }
    }

    pub fn get_block(&self) -> *mut u8 {
        let free_list = self.list.get();
        if !free_list.is_null() {
            let next_block = unsafe { *(free_list as *mut *mut u8) };
            self.list.set(next_block);
        }

        free_list
    }

    pub fn return_block(&self, block: *mut u8) {
            let free_list = self.list.get();
            let returned_ptr = block;
            unsafe {
                *(returned_ptr as *mut *mut u8) = free_list;
            }
            self.list.set(returned_ptr);

    }
}