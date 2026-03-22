use alloc::fmt;

use crate::peripherals::uart::_print;

#[repr(usize)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

#[macro_export]
macro_rules! info {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::log(format_args!($($arg)*), $crate::logger::LogLevel::Info)
    };
}
#[macro_export]
macro_rules! debug {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::log(format_args!($($arg)*), $crate::logger::LogLevel::Debug)
    };
}
#[macro_export]
macro_rules! err  {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::log(format_args!($($arg)*), $crate::logger::LogLevel::Error)
    };
}
#[macro_export]
macro_rules! warn {
    () => {};
    ($($arg:tt)*) => {
        $crate::logger::log(format_args!($($arg)*), $crate::logger::LogLevel::Warn)
    };
}

pub fn log(args: fmt::Arguments, level: LogLevel) {
    _print(args);
}
