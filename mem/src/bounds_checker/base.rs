pub trait BoundsChecker {
    unsafe fn write_canary(&self, memory: *mut u8);
    fn validate_front_canary(&self, memory: *const u8);
    fn validate_back_canary(&self, memory: *const u8);
    fn get_canary(&self) -> u32;
    fn get_canary_size(&self) -> u32;
}