#![no_std]
#![doc = include_str!("../README.md")]

mod alarm;
pub mod future;
pub mod scheduler;
pub mod sharedstate;

// Export the scheduler as it's the main entry point for this crate
pub use scheduler::DelayScheduler;

// Re-export dependency crates
pub extern crate critical_section;
pub extern crate rp2040_hal;
