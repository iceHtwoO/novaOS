use core::fmt::Write;

use nova::logger::Logger;

/// Goals:
/// - I want to have a functional terminal over uart
/// - It shall continue to log

pub struct Terminal {}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(())
    }
}

impl Logger for Terminal {}
