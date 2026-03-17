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

use crate::{
    aarch64::mmu::{
        allocate_memory, KERNEL_VIRTUAL_MEM_SPACE, LEVEL2_BLOCK_SIZE, NORMAL_MEM, UXN, WRITABLE,
    },
    interrupt_handlers::initialize_interrupt_handler,
    logger::DefaultLogger,
};

static PERIPHERAL_BASE: usize = 0x3F00_0000;

unsafe extern "C" {
    unsafe static mut __kernel_end: u8;
}

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: Heap = Heap::empty();

pub unsafe fn init_kernel_heap() {
    let start = core::ptr::addr_of_mut!(__kernel_end) as usize | KERNEL_VIRTUAL_MEM_SPACE;
    let size = LEVEL2_BLOCK_SIZE * 2;

    allocate_memory(start, size, NORMAL_MEM | UXN | WRITABLE).unwrap();
    let heap = core::ptr::addr_of_mut!(GLOBAL_ALLOCATOR);
    (*heap).init(start, start + size);
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        println!("Panic: {}", _panic.message());
    }
}

pub mod peripherals;

pub mod aarch64;
pub mod configuration;
pub mod framebuffer;
pub mod interrupt_handlers;
pub mod logger;

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
    unsafe { init_kernel_heap() };
    logger::set_logger(Box::new(DefaultLogger));
    initialize_interrupt_handler();
}
