#![no_std]
#![allow(clippy::missing_safety_doc)]
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{read_volatile, write_volatile},
};

use heap::Heap;

pub static PERIPHERAL_BASE: u32 = 0x3F00_0000;

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

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::peripherals::uart::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {};
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\r\n");
    };
}

pub mod peripherals;

pub mod configuration;
pub mod framebuffer;
pub mod irq_interrupt;
pub mod mailbox;
pub mod power_management;
pub mod timer;

pub fn mmio_read(address: u32) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

pub fn mmio_write(address: u32, data: u32) {
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
