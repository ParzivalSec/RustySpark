extern crate winapi;

use std::mem;
use std::ptr;
use virtual_mem::winapi::shared::minwindef::{ LPVOID };
use virtual_mem::winapi::um::sysinfoapi;
use virtual_mem::winapi::um::memoryapi::{ VirtualAlloc };
use virtual_mem::winapi::um::winnt;
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
pub fn reserve_virtual_memory(mem_size: usize) -> *mut u8 {
    let raw_mem: *mut u8;

    unsafe {
        let v_alloc_mem: LPVOID = VirtualAlloc(ptr::null_mut(), mem_size, MEM_RESERVE, PAGE_NOACCESS);
        raw_mem = v_alloc_mem as *mut u8;
    }
    
    raw_mem
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_proper_page_size() {
        let page_size: usize = get_page_size();
        assert_eq!(page_size, 4096);
    }

    #[test]
    fn reserve_virtual_address_space() {
        use virtual_mem::winapi::um::memoryapi::{ VirtualQuery };

        let page_size: usize = get_page_size();
        let quadruple_page_size: usize = page_size * 4;

        let v_mem_ptr = reserve_virtual_memory(quadruple_page_size);

        // Invoke VirtualQuery to get information about the region we just reserved before
        let mut region_info: winnt::MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };

        unsafe {
            let region_info_ptr = &mut region_info as winnt::PMEMORY_BASIC_INFORMATION;
            let err = VirtualQuery(v_mem_ptr as LPVOID, region_info_ptr, mem::size_of::<winnt::MEMORY_BASIC_INFORMATION>());
            assert_ne!(0, err);
        }

        // Check whether the whole region is reserved and PAGE_READWRITE protected or not
        assert_eq!(quadruple_page_size, region_info.RegionSize);
        assert_eq!(MEM_RESERVE, region_info.State);
        assert_eq!(PAGE_NOACCESS, region_info.AllocationProtect);
    }
}
