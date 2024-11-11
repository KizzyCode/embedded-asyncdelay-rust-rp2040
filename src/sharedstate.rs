//! Globally shared state types
#![doc(hidden)]

use crate::alarm::HardwareAlarm;
use core::{cell::RefCell, task::Waker};
use critical_section::{CriticalSection, Mutex};
use rp2040_hal::{
    fugit::{MicrosDurationU32, MicrosDurationU64},
    Timer,
};

/// A waker slot
#[derive(Debug)]
pub enum WakerSlot {
    /// The slot is empty
    Empty,
    /// The slot is reserved
    Reserved,
    /// The slot has a pending waker to wake
    Pending { waker: Waker, deadline: MicrosDurationU64 },
}

/// Size-opaque interface for a mutexed global array of waker slots
pub trait GlobalWakerSlots {
    /// Gets mutable access to the waker slots
    fn get_mut(&self, cs: CriticalSection, scope: &mut dyn FnMut(&mut [WakerSlot]));
}
impl<const SLOTS: usize> GlobalWakerSlots for Mutex<RefCell<[WakerSlot; SLOTS]>> {
    fn get_mut(&self, cs: CriticalSection, scope: &mut dyn FnMut(&mut [WakerSlot])) {
        // Lock mutex and provide access to the stored array as slice
        let mut slots = self.borrow_ref_mut(cs);
        scope(slots.as_mut_slice());
    }
}

/// Shared state storage between the scheduler and the IRQ handler
pub struct SharedState {
    /// The periodic alarm interval
    pub interval: MicrosDurationU32,
    /// A timer to get the current monotonic time
    pub timer: Timer,
    /// The alarm peripheral
    pub alarm: HardwareAlarm,
}
impl SharedState {
    /// Allocates a waker slot
    pub fn alloc(&self, cs: CriticalSection) -> Result<usize, &'static str> {
        self.wakers(cs, |slots| {
            // Find the first empty slot
            let (index, slot) = (slots.iter_mut().enumerate())
                .find(|(_, slot)| matches!(slot, WakerSlot::Empty))
                .ok_or("No empty waker slot available")?;

            // Reserve slot
            *slot = WakerSlot::Reserved;
            Ok(index)
        })
    }

    /// Gets mutable access to a waker slot
    pub fn slot<F, R>(&self, cs: CriticalSection, index: usize, scope: F) -> Result<R, &'static str>
    where
        F: FnOnce(&mut WakerSlot) -> R,
    {
        self.wakers(cs, |slots| {
            // Get slot and pass it to the scope
            let slot = slots.get_mut(index).ok_or("Invalid slot index")?;
            let result = scope(slot);
            Ok(result)
        })
    }

    /// Scoped access to the waker slots
    pub fn wakers<F, R>(&self, cs: CriticalSection, scope: F) -> R
    where
        F: FnOnce(&mut [WakerSlot]) -> R,
    {
        // Dependency injection for global waker slots
        extern "Rust" {
            fn _asyncdelay_wakerslots_pN7uDuTu() -> &'static dyn GlobalWakerSlots;
        }

        // Get slot and create letterboxes for FnMut
        let slots = unsafe { _asyncdelay_wakerslots_pN7uDuTu() };
        let mut scope = Some(scope);
        let mut result = None;

        // Access slots via FnMut and move scope in and result out
        slots.get_mut(cs, &mut |slots| {
            // Take and call closure and store result
            let scope = scope.take().expect("scope function is missing");
            result = Some(scope(slots));
        });

        // Return result
        result.expect("scope callback has not been called")
    }

    /// The globally shared state
    #[inline]
    pub fn global() -> &'static Mutex<RefCell<Option<SharedState>>> {
        static SHARED: Mutex<RefCell<Option<SharedState>>> = Mutex::new(RefCell::new(None));
        &SHARED
    }
}

/// Allocates space on the stack for the waker slots
///
/// # Note
/// This macro declares a `pub extern "Rust"`-function and must be called on module level/top level, and not from within
/// a function. It must only be used if the `init*`-features of this crate are disabled, and only once; otherwise it
/// will yield a rather cryptic linker error and compilation will fail.
#[cfg_attr(not(feature = "init16"), macro_export)]
macro_rules! setup_waker_slots {
    ($wakers_max:expr) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "Rust" fn _asyncdelay_wakerslots_pN7uDuTu() -> &'static dyn $crate::sharedstate::GlobalWakerSlots {
            use core::cell::RefCell;
            use critical_section::Mutex;
            use $crate::sharedstate::WakerSlot;

            /// Default value for const array initialization
            const WAKER_SLOT_DEFAULT: WakerSlot = WakerSlot::Empty;
            /// The waker slots
            static WAKER_SLOTS: Mutex<RefCell<[WakerSlot; $wakers_max]>> =
                Mutex::new(RefCell::new([WAKER_SLOT_DEFAULT; $wakers_max]));
            &WAKER_SLOTS
        }
    };
}

// Initialize the waker slots with a default size
#[cfg(feature = "init16")]
setup_waker_slots!(16);
