#![macro_use]
/// Debugging utilities

use crate::fixed::Fixed;

use gba::io::timers::TM0CNT_L;

/// Dump a message to mGBA's debug console, if possible.
macro_rules! spew (
    () => {};
    ($($arg:tt)*) => ({
        use gba::mgba::{MGBADebug, MGBADebugLevel};
        use core::fmt::Write;
        if let Some(mut debug) = MGBADebug::new() {
            match write!(debug, $($arg)*) {
                Ok(_) => {
                    debug.send(MGBADebugLevel::Debug);
                }
                Err(e) => {
                    if write!(debug, "<spew error: {:?}>", e).is_ok() {
                        debug.send(MGBADebugLevel::Debug);
                    }
                }
            }
        }
    });
);


/// Prints the time elapsed through the current frame (as a percentage of one frame) with a
/// message.
macro_rules! spew_time (
    ($arg:expr) => ({
        spew!("{} at {:?}", $arg, Fixed::promote(TM0CNT_L.read() as i16) * 100 / 4389);
    });
);



/// Prints the time elapsed (as a percentage of one frame) when dropped.
///
/// Assumes timer 0 has been set up to run at speed 64.
pub struct StopwatchGuard {
    start: u16,
    message: Option<&'static str>,
}

impl StopwatchGuard {
    pub fn new() -> Self {
        StopwatchGuard{ start: TM0CNT_L.read(), message: None }
    }

    pub fn with_message(message: &'static str) -> Self {
        StopwatchGuard{ start: TM0CNT_L.read(), message: Some(message) }
    }
}

impl Drop for StopwatchGuard {
    fn drop(&mut self) {
        let dt = TM0CNT_L.read() - self.start;
        spew!("{} took {:?}", self.message.unwrap_or("???"), Fixed::promote(dt as i16) * 100 / 4389);
    }
}
