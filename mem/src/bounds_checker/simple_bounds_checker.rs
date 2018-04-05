use std;
use super::base::{ BoundsChecker };

///
/// SimpleBoundsChecker can write a marker value at the specified memory location
/// and has the capabilities to verify the canary markers again for a given memory
/// location
///
pub struct SimpleBoundsChecker {
    canary: u32,
}

impl Default for SimpleBoundsChecker {
    fn default() -> SimpleBoundsChecker {
        SimpleBoundsChecker {
            canary: 0xCA,
        }
    }
}

impl BoundsChecker for SimpleBoundsChecker {
    unsafe fn write_canary(&self, memory: *mut u8) {
        std::ptr::write(memory as *mut u32, self.canary);
    }

    fn validate_front_canary(&self, memory: *const u8) {
        if !memory.is_null() {
            let marker = unsafe { std::ptr::read(memory as *const u32) };
            let is_valid_canary = marker == self.canary;
            debug_assert!(is_valid_canary, "Front canary was not valid");
        }
    }

    fn validate_back_canary(&self, memory: *const u8) {
        if !memory.is_null() {
            let marker = unsafe { std::ptr::read(memory as *const u32) };
            let is_valid_canary = marker == self.canary;
            debug_assert!(is_valid_canary, "Back canary was not valid");
        }
    }

    fn get_canary(&self) -> u32 {
        self.canary
    }

    fn get_canary_size(&self) -> u32 {
        std::mem::size_of::<u32>() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_write_canary() {
        let bounds_checker: SimpleBoundsChecker = Default::default();
        
        let memory = &mut [50; 50];
        let ptr = memory.as_mut_ptr();

        unsafe { 
            bounds_checker.write_canary(ptr);
            let marker: u32 = *(ptr as *mut u32);
            assert_eq!(marker, bounds_checker.get_canary());
        };
    }

    #[test]
    fn can_validate_front_canary() {
        let bounds_checker: SimpleBoundsChecker = Default::default();
        let memory = &mut [50; 50];
        let ptr = memory.as_mut_ptr();

        unsafe { bounds_checker.write_canary(ptr); }

        bounds_checker.validate_front_canary(ptr);
    }

    #[test]
    fn can_validate_back_canary() {
        let bounds_checker: SimpleBoundsChecker = Default::default();
        let memory = &mut [50; 50];
        let ptr = unsafe { memory.as_mut_ptr().offset(46) };

        unsafe { bounds_checker.write_canary(ptr); }

        bounds_checker.validate_back_canary(ptr);
    }

    #[test]
    #[should_panic(expected = "Front canary was not valid")]
    fn shall_panic_on_corrupt_front_canary() {
        let bounds_checker: SimpleBoundsChecker = Default::default();
        let memory = &mut [50; 50];
        let ptr = memory.as_mut_ptr();

        unsafe { 
            bounds_checker.write_canary(ptr); 
            std::ptr::write(ptr as *mut u32, 0xCC); // Simulate a memory stomp
        }

        bounds_checker.validate_front_canary(ptr);
    }

    #[test]
    #[should_panic(expected = "Back canary was not valid")]
    fn shall_panic_on_corrupt_back_canary() {
        let bounds_checker: SimpleBoundsChecker = Default::default();
        let memory = &mut [50; 50];
        let ptr = unsafe { memory.as_mut_ptr().offset(46) };

        unsafe { 
            bounds_checker.write_canary(ptr); 
            std::ptr::write(ptr as *mut u32, 0xCC); // Simulate a memory stomp
        }

        bounds_checker.validate_back_canary(ptr);
    }
}