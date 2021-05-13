use futures_intrusive::sync::GenericMutex;
use lock_api::{RawMutex, GuardSend};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct RawSpinlock(AtomicBool);

// taken from lock_api example
unsafe impl RawMutex for RawSpinlock {
    const INIT: RawSpinlock = RawSpinlock(AtomicBool::new(false));

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        // Note: This isn't the best way of implementing a spinlock, but it
        // suffices for the sake of this example.
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        self.0
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

// 3. Export the wrappers. This are the types that your users will actually use.
pub type Mutex<T> = GenericMutex<RawSpinlock, T>;
