//! A module to make the underlying hardware alarm compile-time configurable via feature flags

use crate::sharedstate::{SharedState, WakerSlot};
use rp2040_hal::timer::Alarm;

/// The interrupt handler implementation for alarm
#[inline]
fn irq_handler() {
    critical_section::with(|cs| {
        // Get shared state
        let mut shared_slot = SharedState::global().borrow(cs).borrow_mut();
        let shared = shared_slot.as_mut().expect("failed to access shared state");

        // Reschedule alarm
        shared.alarm.clear_interrupt();
        shared.alarm.schedule(shared.interval).expect("failed to reschedule alarm");

        // Wake all ready wakers
        let now = shared.timer.get_counter().duration_since_epoch();
        shared.wakers(cs, |slots| {
            // Iterate over all slots
            for slot in slots {
                if let WakerSlot::Pending { waker, deadline } = slot {
                    // Check the deadline
                    if now >= *deadline {
                        // Wake the waker and mark the slot as reserved again
                        waker.wake_by_ref();
                    }
                }
            }
        })
    })
}

// Configure Alarm0 as the underlying hardware alarm
#[cfg(feature = "alarm0")]
mod alarm_cfg {
    use crate::alarm::irq_handler;
    use rp2040_hal::{
        pac::{interrupt, Interrupt},
        timer::Alarm0,
    };

    /// The configured hardware alarm
    pub type HardwareAlarm = Alarm0;
    /// The associated IRQ for the configured hardware alarm
    pub const TIMER_IRQ: Interrupt = interrupt::TIMER_IRQ_0;

    #[interrupt]
    fn TIMER_IRQ_0() {
        irq_handler();
    }
}

// Configure Alarm1 as the underlying hardware alarm
#[cfg(feature = "alarm1")]
mod alarm_cfg {
    use crate::alarm::irq_handler;
    use rp2040_hal::{
        pac::{interrupt, Interrupt},
        timer::Alarm1,
    };

    /// The configured hardware alarm
    pub type HardwareAlarm = Alarm1;
    /// The associated IRQ for the configured hardware alarm
    pub const TIMER_IRQ: Interrupt = interrupt::TIMER_IRQ_1;

    #[interrupt]
    fn TIMER_IRQ_1() {
        irq_handler();
    }
}

// Configure Alarm2 as the underlying hardware alarm
#[cfg(feature = "alarm2")]
mod alarm_cfg {
    use crate::alarm::irq_handler;
    use rp2040_hal::{
        pac::{interrupt, Interrupt},
        timer::Alarm2,
    };

    /// The configured hardware alarm
    pub type HardwareAlarm = Alarm2;
    /// The associated IRQ for the configured hardware alarm
    pub const TIMER_IRQ: Interrupt = interrupt::TIMER_IRQ_2;

    #[interrupt]
    fn TIMER_IRQ_2() {
        irq_handler();
    }
}

// Configure Alarm3 as the underlying hardware alarm
#[cfg(feature = "alarm3")]
mod alarm_cfg {
    use crate::alarm::irq_handler;
    use rp2040_hal::{
        pac::{interrupt, Interrupt},
        timer::Alarm3,
    };

    /// The configured hardware alarm
    pub type HardwareAlarm = Alarm3;
    /// The associated IRQ for the configured hardware alarm
    pub const TIMER_IRQ: Interrupt = interrupt::TIMER_IRQ_3;

    #[interrupt]
    fn TIMER_IRQ_3() {
        irq_handler();
    }
}

// Raise a human-readable compile error if no alarm-feature is configured
#[cfg(not(any(feature = "alarm0", feature = "alarm1", feature = "alarm2", feature = "alarm3")))]
compile_error!("You must define the hardware alarm to use via the appropriate cargo config feature");

// Re-export the selected configurations
pub use alarm_cfg::{HardwareAlarm, TIMER_IRQ};
