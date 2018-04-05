use super::base::{ BoundsChecker };

///
/// The EmptyBoundsChecker is a simple abstraction and every functions yields a no-op
/// This type is used to disable bounds-checking in release/retail configurations by
/// simple changing the type of the bounds checker in action to this one
///
pub struct EmptyBoundsChecker {}

impl BoundsChecker for EmptyBoundsChecker {
    unsafe fn write_canary(&self, _memory: *mut u8) {}
    fn validate_front_canary(&self, _memory: *const u8) {}
    fn validate_back_canary(&self, _memory: *const u8) {}
    fn get_canary(&self) -> u32 { 0 }
    fn get_canary_size(&self) -> u32 { 0 }
}

