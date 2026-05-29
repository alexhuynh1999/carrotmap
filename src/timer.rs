use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};

/// A RAII guard that sets the Windows scheduler timer resolution to 1ms
/// for the lifetime of the guard, and restores it on drop.
pub struct TimerResolutionGuard;

impl TimerResolutionGuard {
    pub fn new() -> Self {
        unsafe {
            let _ = timeBeginPeriod(1);
        }
        TimerResolutionGuard
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = timeEndPeriod(1);
        }
    }
}
