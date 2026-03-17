use core::fmt::Write;

use alloc::{boxed::Box, fmt};

use crate::peripherals::uart;

static mut LOGGER: Option<Box<dyn Logger>> = None;

pub trait Logger: Write + Sync {
    fn flush(&mut self);
}

pub struct DefaultLogger;

impl Logger for DefaultLogger {
    fn flush(&mut self) {}
}

impl Write for DefaultLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart::Uart.write_str(s)
    }
}

#[macro_export]
macro_rules! log {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::log(format_args!($($arg)*))
    };
}

pub fn log(args: fmt::Arguments) {
    if let Some(logger) = unsafe { &mut *core::ptr::addr_of_mut!(LOGGER) } {
        logger.write_str("\n").unwrap();
        logger.write_fmt(args).unwrap();
        logger.flush();
    }
}

pub fn set_logger(logger: Box<dyn Logger>) {
    unsafe {
        LOGGER = Some(logger);
    }
}
