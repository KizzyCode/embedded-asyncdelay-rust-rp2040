#![no_std]
#![doc = include_str!("../README.md")]

mod alarm;
pub mod future;
pub mod scheduler;
mod sharedstate;

// Export the scheduler as it's the main entry point for this crate
pub use scheduler::DelayScheduler;

// Re-export dependency crates
pub extern crate critical_section;
pub extern crate rp2040_hal;

// Re-export private modules if necessary
#[cfg(not(feature = "init16"))]
#[doc(hidden)]
pub use crate::sharedstate::{GlobalWakerSlots, WakerSlot};
