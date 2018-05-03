use std;
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::profileapi::{ QueryPerformanceCounter, QueryPerformanceFrequency };

pub struct HighPrecisionClock
{
    pub start: i64,
    pub frequency: f64,
}

impl HighPrecisionClock {
    pub unsafe fn new() -> Self {
        let mut freq: LARGE_INTEGER = std::mem::uninitialized();
        QueryPerformanceFrequency(&mut freq);
        HighPrecisionClock {
            start: 0,
            frequency: 1.0 / (*freq.QuadPart() as f64 / 1000000.0),
        }
    }

    pub unsafe fn start(&mut self) {
        let mut cycles: LARGE_INTEGER = std::mem::uninitialized();
        QueryPerformanceCounter(&mut cycles);
        self.start = *cycles.QuadPart();
    }

    pub unsafe fn get(&self) -> f64 {
        let mut curr_cycles: LARGE_INTEGER = std::mem::uninitialized();
        QueryPerformanceCounter(&mut curr_cycles);
        (*curr_cycles.QuadPart() as f64 - self.start as f64) * self.frequency
    }
}