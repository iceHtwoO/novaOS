use core::fmt::Write;

use alloc::{boxed::Box, fmt};

use crate::peripherals::uart;

static mut LOGGER: Option<Box<dyn Logger>> = None;

pub trait Logger: Write + Sync {}

pub struct DefaultLogger;

impl Logger for DefaultLogger {}

impl Write for DefaultLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart::write_str(s)
    }
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::_print(format_args!($($arg)*))
    };
}

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(logger) = LOGGER.as_mut() {
            logger.write_fmt(args);
        }
    }
}

#[macro_export]
macro_rules! println {
    () => {};
    ($($arg:tt)*) => {
        $crate::print!($($arg)*);
        $crate::print!("\r\n");
    };
}

pub fn set_logger(logger: Box<dyn Logger>) {
    unsafe {
        LOGGER = Some(logger);
    }
}
