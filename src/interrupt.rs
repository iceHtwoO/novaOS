use core::{
    arch::asm,
    ptr::{read_volatile, write_volatile},
};

use crate::uart::print;

const INTERRUPT_BASE: u32 = 0x3F00_B000;
const ENABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x210;
const DISABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x21C;

#[no_mangle]
pub unsafe extern "C" fn irq_handler() {
    print("Interrupt\r\n");
}

pub fn enable_iqr_source(nr: u32) {
    let register = ENABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    unsafe {
        let current = read_volatile(register as *const u32);
        let mask = 0b1 << register_offset;
        let new_val = current | mask;
        write_volatile(register as *mut u32, new_val);
    }
}

pub fn disable_iqr_source(nr: u32) {
    let register = DISABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    unsafe {
        let current = read_volatile(register as *const u32);
        let mask = 0b1 << register_offset;
        let new_val = current | mask;
        write_volatile(register as *mut u32, new_val);
    }
}
