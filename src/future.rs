//! A delay future that yields until the given time has elapsed

use crate::sharedstate::{SharedState, WakerSlot};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use rp2040_hal::{fugit::MicrosDurationU64, Timer};

/// A delay future that yields until the given time has elapsed
pub struct DelayFuture {
    /// The deadline for the future
    pub(crate) deadline: MicrosDurationU64,
    /// The timer peripheral
    pub(crate) timer: Timer,
    /// The waker slot index
    pub(crate) slot: usize,
}
impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Check if we are done
        let now = self.timer.get_counter().duration_since_epoch();
        if now >= self.deadline {
            // We are done now
            return Poll::Ready(());
        }

        // Store the waker
        let waker = cx.waker().clone();
        critical_section::with(|cs| {
            // Get share state
            let mut shared_slot = SharedState::global().borrow(cs).borrow_mut();
            let shared = shared_slot.as_mut().expect("failed to access shared state");

            // Set our slot
            (shared.slot(cs, self.slot, |slot| *slot = WakerSlot::Pending { waker, deadline: self.deadline }))
                .expect("invalid waker slot index");
        });

        // We are not done yet
        Poll::Pending
    }
}
impl Drop for DelayFuture {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            // Get share state
            let mut shared_slot = SharedState::global().borrow(cs).borrow_mut();
            let shared = shared_slot.as_mut().expect("failed to access shared state");

            // Free our slot
            shared.slot(cs, self.slot, |slot| *slot = WakerSlot::Empty).expect("invalid waker slot index");
        });
    }
}
