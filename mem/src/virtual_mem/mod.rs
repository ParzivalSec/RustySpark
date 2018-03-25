extern crate winapi;

use std::mem;
use std::ptr;
use virtual_mem::winapi::shared::minwindef::{ LPVOID };
use virtual_mem::winapi::um::sysinfoapi;
use virtual_mem::winapi::um::memoryapi::{ VirtualAlloc, VirtualFree };
use virtual_mem::winapi::um::winnt::{ MEM_COMMIT, MEM_RESERVE, MEM_DECOMMIT, MEM_RELEASE, PAGE_READWRITE, PAGE_NOACCESS};

#[cfg(windows)]
pub fn get_page_size() -> usize {
    let mut sys_info: sysinfoapi::SYSTEM_INFO = unsafe { mem::zeroed() };
    
    unsafe {
        let info_ptr: sysinfoapi::LPSYSTEM_INFO = &mut sys_info as sysinfoapi::LPSYSTEM_INFO;
        sysinfoapi::GetSystemInfo(info_ptr);
    }

    return sys_info.dwPageSize as usize; 
}

#[cfg(windows)]
pub fn reserve_address_space(mem_size: usize) -> Option<*mut u8> {
    let raw_mem: *mut u8;

    unsafe {
        let v_alloc_mem: LPVOID = VirtualAlloc(ptr::null_mut(), mem_size, MEM_RESERVE, PAGE_NOACCESS);
        raw_mem = v_alloc_mem as *mut u8;
    }
    
    if raw_mem != ptr::null_mut() 
    {
        Some(raw_mem)
    }
    else 
    {
        None
    }
}

#[cfg(windows)]
pub fn commit_physical_memory(base_address: *mut u8, mem_size: usize) -> Option<*mut u8> {
        let physical_mem: *mut u8;

    unsafe {
        let v_alloc_mem: LPVOID = VirtualAlloc(base_address as LPVOID, mem_size, MEM_COMMIT, PAGE_READWRITE);
        physical_mem = v_alloc_mem as *mut u8;
    }
    
    if physical_mem != ptr::null_mut() 
    {
        Some(physical_mem)
    }
    else 
    {
        None
    }
}

#[cfg(windows)]
pub fn decommit_physical_memory(base_address: *mut u8, mem_size: usize) {
    unsafe {
        VirtualFree(base_address as LPVOID, mem_size, MEM_DECOMMIT);
    }
}

#[cfg(windows)]
pub fn free_address_space(base_address: *mut u8) {
    unsafe {
        VirtualFree(base_address as LPVOID, 0usize, MEM_RELEASE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_mem::winapi::um::winnt::{ MEMORY_BASIC_INFORMATION, PMEMORY_BASIC_INFORMATION, MEM_FREE };
    use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

    #[test]
    fn ensure_proper_page_size() {
        let page_size: usize = get_page_size();
        assert_eq!(page_size, 4096);
    }

    #[test]
    fn reserve_virtual_address_space() {

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;

        let v_mem_ptr = reserve_address_space(quadruple_page_size).unwrap();

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(v_mem_ptr as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(quadruple_page_size, region_info.RegionSize);
        assert_eq!(MEM_RESERVE, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
    }

    #[test]
    fn commit_physical_address_space() {
        use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;

        let v_mem_ptr = reserve_address_space(quadruple_page_size).unwrap();
        let p_mem_ptr = commit_physical_memory(v_mem_ptr, quadruple_page_size).unwrap();

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(p_mem_ptr as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(quadruple_page_size, region_info.RegionSize);
        assert_eq!(MEM_COMMIT, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
        assert_eq!(PAGE_READWRITE, region_info.Protect);
    }

    #[test]
    fn decommit_physical_address_space() {
        use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;

        let v_mem_ptr = reserve_address_space(quadruple_page_size).unwrap();
        let p_mem_ptr = commit_physical_memory(v_mem_ptr, quadruple_page_size).unwrap();
        decommit_physical_memory(p_mem_ptr, quadruple_page_size);

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(p_mem_ptr as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(quadruple_page_size, region_info.RegionSize);
        assert_eq!(MEM_RESERVE, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
    }

    #[test]
    fn free_reserved_address_space() {
        use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;

        let v_mem_ptr = reserve_address_space(quadruple_page_size).unwrap();
        let p_mem_ptr = commit_physical_memory(v_mem_ptr, quadruple_page_size).unwrap();
        free_address_space(v_mem_ptr);

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(p_mem_ptr as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(MEM_FREE, region_info.State);
    }

    #[test]
    fn commit_physical_address_space_multiple_times() {
        use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;
        let double_page_size: usize = page_size * 2;

        let v_mem_ptr = reserve_address_space(quadruple_page_size).unwrap();
        let p_mem_ptr_0 = commit_physical_memory(v_mem_ptr, double_page_size).unwrap();

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(p_mem_ptr_0 as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(double_page_size, region_info.RegionSize);
        assert_eq!(MEM_COMMIT, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
        assert_eq!(PAGE_READWRITE, region_info.Protect);
            
        unsafe {
            let p_mem_ptr_1 = commit_physical_memory(p_mem_ptr_0.offset(double_page_size as isize), double_page_size).unwrap();
            let region_info_ptr = &mut region_info as PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(p_mem_ptr_1 as LPVOID, region_info_ptr, mem::size_of::<MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        assert_eq!(double_page_size, region_info.RegionSize);
        assert_eq!(MEM_COMMIT, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
        assert_eq!(PAGE_READWRITE, region_info.Protect);
    }
}