//! A scheduler to create delay futures

use crate::{
    alarm::{HardwareAlarm, TIMER_IRQ},
    future::DelayFuture,
    sharedstate::SharedState,
};
use core::time::Duration;
use rp2040_hal::{
    fugit::{MicrosDurationU32, MicrosDurationU64},
    pac::NVIC,
    timer::Alarm,
    Timer,
};

/// A scheduler to create alarm futures
#[derive(Clone, Copy)]
pub struct DelayScheduler {
    /// The timer peripheral
    timer: Timer,
}
impl DelayScheduler {
    /// Initialize the delay scheduler
    ///
    /// # About Resolution
    /// The resolution defines how often the scheduler will wakeup to see if there are some futures to wake. The
    /// resolution is a tradeoff between accuracy and load. A higher resolution means more accurate wakeups, but also
    /// increases load and power consumption on the CPU.
    pub fn new(mut alarm: HardwareAlarm, timer: Timer, resolution: Duration) -> Self {
        // Convert duration into internal representation
        let interval = u32::try_from(resolution.as_millis()).expect("resolution is too low");
        let interval = MicrosDurationU32::millis(interval);

        // Enable interrupts
        alarm.enable_interrupt();
        unsafe { NVIC::unmask(TIMER_IRQ) };

        // Setup periodic alarm
        critical_section::with(|cs| {
            // Schedule alarm and store the required IRQ state
            alarm.schedule(interval).expect("failed to schedule alarm");

            // Initialize the shared state
            let shared = SharedState { alarm, interval, timer };
            SharedState::global().borrow(cs).replace(Some(shared));
        });

        // Init self
        Self { timer }
    }

    /// Creates a future that yields for the given duration
    ///
    /// # Panics
    /// This function panics if there is no free slot for a future/waker available anymore.
    pub fn schedule(&self, timeout: Duration) -> DelayFuture {
        // Allocate slot
        let slot_index = critical_section::with(|cs| {
            // Get the shared state
            let mut shared_slot = SharedState::global().borrow(cs).borrow_mut();
            let shared = shared_slot.as_mut().expect("failed to access shared state");

            // Allocate the slot
            shared.alloc(cs).expect("failed to allocate waker slot")
        });

        // Convert duration into internal representation
        let timeout = u64::try_from(timeout.as_millis()).expect("timeout is too large");
        let timeout = MicrosDurationU64::millis(timeout);

        // Compute deadline and create future
        let now = self.timer.get_counter().duration_since_epoch();
        let deadline = now.checked_add(timeout).expect("timeout is too large");
        DelayFuture { deadline, timer: self.timer, slot: slot_index }
    }
}
