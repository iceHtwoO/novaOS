#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::boxed::Box;
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{read_volatile, write_volatile},
};

use heap::Heap;

use crate::{interrupt_handlers::initialize_interrupt_handler, logger::DefaultLogger};

static PERIPHERAL_BASE: u32 = 0x3F00_0000;

unsafe extern "C" {
    unsafe static mut __heap_start: u8;
    unsafe static mut __heap_end: u8;
}

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: Heap = Heap::empty();

pub unsafe fn init_heap() {
    let start = core::ptr::addr_of_mut!(__heap_start) as usize;
    let end = core::ptr::addr_of_mut!(__heap_end) as usize;

    let heap = core::ptr::addr_of_mut!(GLOBAL_ALLOCATOR);
    (*heap).init(start, end);
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        println!("Panic");
    }
}

pub mod peripherals;

pub mod aarch64;
pub mod configuration;
pub mod framebuffer;
pub mod interrupt_handlers;
pub mod logger;
pub mod timer;

pub mod pi3;

#[inline(always)]
pub unsafe fn read_address(address: u32) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

#[inline(always)]
pub unsafe fn write_address(address: u32, data: u32) {
    unsafe { write_volatile(address as *mut u32, data) }
}

pub fn get_current_el() -> u64 {
    let el: u64;
    unsafe {
        asm!(
            "mrs {el}, CurrentEL",
            el = out(reg) el,
            options(nomem, nostack, preserves_flags)
        );
    }
    el >> 2
}

pub fn initialize_kernel() {
    logger::set_logger(Box::new(DefaultLogger));
    initialize_interrupt_handler();
}
