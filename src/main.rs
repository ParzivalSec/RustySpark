#![feature(alloc_system, allocator_api)]

extern crate libc;
extern crate mem;
extern crate container;
extern crate alloc_system;
extern crate time;
extern crate spark_core;

// Calx-ECS
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate calx_ecs;

mod scenarios;

use std::env;
use spark_core::clock::HighPrecisionClock;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        println!("Usage: benchmark.exe SCENARIO_ID");
        return;
    }

    unsafe {
        let mut clock = HighPrecisionClock::new();

        clock.start();
        scenarios::SCENARIOS[arguments[1].parse::<usize>().expect("Could not parse arg")]();
        println!("{:.3}", clock.get());
    }
}
