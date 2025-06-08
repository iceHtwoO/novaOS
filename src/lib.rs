#![no_std]

use core::ptr::{read_volatile, write_volatile};

pub mod peripherals;

pub mod irq_interrupt;
pub mod mailbox;
pub mod timer;

pub fn mmio_read(address: u32) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

pub fn mmio_write(address: u32, data: u32) {
    unsafe { write_volatile(address as *mut u32, data) }
}
