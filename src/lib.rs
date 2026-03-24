#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{read_volatile, write_volatile},
};
use log::LevelFilter;
use log::{Level, Metadata, Record};

use heap::Heap;

use crate::{
    aarch64::mmu::{
        allocate_memory, PhysSource, KERNEL_VIRTUAL_MEM_SPACE, LEVEL2_BLOCK_SIZE, NORMAL_MEM, UXN,
        WRITABLE,
    },
    interrupt_handlers::irq::initialize_interrupt_handler,
    pi3::timer::sleep_s,
    terminal::{flush_terminal, init_terminal},
};

static LOGGER: UartLogger = UartLogger;
static PERIPHERAL_BASE: usize = 0x3F00_0000;

unsafe extern "C" {
    unsafe static mut __kernel_end: u8;
}

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: Heap = Heap::empty();

pub unsafe fn initialize_kernel_heap() {
    let start = core::ptr::addr_of_mut!(__kernel_end) as usize | KERNEL_VIRTUAL_MEM_SPACE;
    let size = LEVEL2_BLOCK_SIZE * 2;

    allocate_memory(start, size, PhysSource::Any, NORMAL_MEM | UXN | WRITABLE).unwrap();
    let heap = core::ptr::addr_of_mut!(GLOBAL_ALLOCATOR);
    (*heap).init(start, start + size);
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        println!("Panic: {}", _panic.message());
        sleep_s(1);
    }
}

pub mod peripherals;

pub mod aarch64;
pub mod configuration;
pub mod framebuffer;
pub mod interrupt_handlers;

pub mod pi3;
pub mod terminal;

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
    unsafe { initialize_kernel_heap() };
    initialize_interrupt_handler();
    init_terminal();
}

struct UartLogger;

impl log::Log for UartLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            if record.level() <= Level::Info {
                flush_terminal();
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
}
