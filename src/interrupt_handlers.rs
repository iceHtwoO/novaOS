use core::arch::asm;

use crate::{
    aarch64::registers::{daif::mask_all, read_esr_el1, read_exception_source_el},
    get_current_el,
};
use log::debug;

const INTERRUPT_BASE: u32 = 0x3F00_B000;
const IRQ_PENDING_BASE: u32 = INTERRUPT_BASE + 0x204;
const ENABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x210;
const DISABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x21C;

const GPIO_PENDING_BIT_OFFSET: u64 = 0b1111 << 49;

#[repr(C)]
pub struct TrapFrame {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x29: u64,
    pub x30: u64,
}

/// Representation of the ESR_ELx registers
///
///  Reference: D1.10.4
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct EsrElX {
    ec: u32,
    il: u32,
    iss: u32,
}

impl From<u32> for EsrElX {
    fn from(value: u32) -> Self {
        Self {
            ec: value >> 26,
            il: (value >> 25) & 0b1,
            iss: value & 0x1FFFFFF,
        }
    }
}

pub mod irq;
pub mod synchronous;

#[no_mangle]
unsafe extern "C" fn rust_synchronous_interrupt_no_el_change() {
    mask_all();

    let source_el = read_exception_source_el() >> 2;
    debug!("--------Sync Exception in EL{}--------", source_el);
    debug!("No EL change");
    debug!("Current EL: {}", get_current_el());
    debug!("{:?}", EsrElX::from(read_esr_el1()));
    debug!("Return register address: {:#x}", read_esr_el1());
    debug!("-------------------------------------");
}

fn set_return_to_kernel_loop() {
    unsafe {
        asm!("ldr x0, =kernel_loop", "msr ELR_EL1, x0");
        asm!("mov x0, #(0b0101)", "msr SPSR_EL1, x0");
    }
}
