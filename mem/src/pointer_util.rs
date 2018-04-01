///
/// Utility functions to check whether a number is a power of two or not
///
pub fn is_pot(num: usize) -> bool {
    match num {
        0 => false,
        n => (n & (n - 1)) == 0
    }
}

///
/// Utility function for checking whether a pointer is aligned to an 
/// specified alignment or not.
///
pub fn is_aligned_to(address: *const u8, alignment: usize) -> bool {
    let ptr_address = address as usize;
    alignment == 0 || (ptr_address & (alignment - 1)) == 0
}

///
/// Aligns a supplied pointer to the next byte bundary matching alignment
/// This function does not mutate the pointer passed to it
///
pub fn align_top(address: *const u8, alignment: usize) -> *const u8 {
    debug_assert!(is_pot(alignment), "Alignment needs to be a power of two");
    
    let mut ptr_address = address as usize;
    ptr_address += alignment - 1;
    ptr_address &= !(alignment - 1);

    ptr_address as *const u8
}

///
/// Aligns the pointer to the next lower byte boundary matching the 
/// alignment. This functions does not mutate the pointer passed to it.
///
pub fn align_bottom(address: *const u8, alignment: usize) -> *const u8 {
    debug_assert!(is_pot(alignment), "Alignment needs to be a power of two");
    
    let mut ptr_address = address as usize;
    ptr_address &= !(alignment - 1);

    ptr_address as *const u8
}