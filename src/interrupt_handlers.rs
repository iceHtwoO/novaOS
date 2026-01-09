use core::arch::asm;

use crate::{
    get_current_el,
    interrupt_handlers::daif::unmask_irq,
    peripherals::gpio::{read_gpio_event_detect_status, reset_gpio_event_detect_status},
    println, read_address, write_address,
};

const INTERRUPT_BASE: u32 = 0x3F00_B000;
const IRQ_PENDING_BASE: u32 = INTERRUPT_BASE + 0x204;
const ENABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x210;
const DISABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x21C;

const GPIO_PENDING_BIT_OFFSET: u64 = 0b1111 << 49;

#[repr(u32)]
pub enum IRQSource {
    AuxInt = 29,
    I2cSpiSlvInt = 44,
    Pwa0 = 45,
    Pwa1 = 46,
    Smi = 48,
    GpioInt0 = 49,
    GpioInt1 = 50,
    GpioInt2 = 51,
    GpioInt3 = 52,
    I2cInt = 53,
    SpiInt = 54,
    PcmInt = 55,
    UartInt = 57,
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

#[no_mangle]
unsafe extern "C" fn rust_irq_handler() {
    daif::mask_all();
    let pending_irqs = get_irq_pending_sources();

    if pending_irqs & GPIO_PENDING_BIT_OFFSET != 0 {
        handle_gpio_interrupt();
    }
    let source_el = get_exception_return_exception_level() >> 2;
    println!("Source EL: {}", source_el);
    println!("Current EL: {}", get_current_el());
    println!("Return register address: {:#x}", get_elr_el1());
}

#[no_mangle]
unsafe extern "C" fn rust_synchronous_interrupt_no_el_change() {
    daif::mask_all();

    let source_el = get_exception_return_exception_level() >> 2;
    println!("--------Sync Exception in EL{}--------", source_el);
    println!("No EL change");
    println!("Current EL: {}", get_current_el());
    println!("{:?}", EsrElX::from(get_esr_el1()));
    println!("Return register address: {:#x}", get_elr_el1());
    println!("-------------------------------------");
}

/// Synchronous Exception Handler
///
/// Lower Exception level, where the implemented level
/// immediately lower than the target level is using
/// AArch64.
#[no_mangle]
unsafe extern "C" fn rust_synchronous_interrupt_imm_lower_aarch64() {
    daif::mask_all();

    let source_el = get_exception_return_exception_level() >> 2;
    println!("--------Sync Exception in EL{}--------", source_el);
    println!("Exception escalated to EL {}", get_current_el());
    println!("Current EL: {}", get_current_el());
    let esr = EsrElX::from(get_esr_el1());
    println!("{:?}", EsrElX::from(esr));
    println!("Return register address: {:#x}", get_elr_el1());

    match esr.ec {
        0b100100 => {
            println!("Cause: Data Abort from a lower Exception level");
        }
        _ => {}
    }
    println!("-------------------------------------");

    set_return_to_kernel_main();
}

fn set_return_to_kernel_main() {
    unsafe {
        asm!("ldr x0, =kernel_main", "msr ELR_EL1, x0");
        asm!("mov x0, #(0b0101)", "msr SPSR_EL1, x0");
    }
}

fn get_exception_return_exception_level() -> u32 {
    let spsr: u32;
    unsafe {
        asm!("mrs {0:x}, SPSR_EL1", out(reg) spsr);
    }
    spsr & 0b1111
}

/// Read the syndrome information that caused an exception
///
/// ESR = Exception Syndrome Register
fn get_esr_el1() -> u32 {
    let esr: u32;
    unsafe {
        asm!(
            "mrs {esr:x}, ESR_EL1",
            esr = out(reg) esr
        );
    }
    esr
}

/// Read the return address
///
/// ELR = Exception Link Registers
fn get_elr_el1() -> u32 {
    let elr: u32;
    unsafe {
        asm!(
            "mrs {esr:x}, ELR_EL1",
            esr = out(reg) elr
        );
    }
    elr
}

fn handle_gpio_interrupt() {
    println!("Interrupt");
    for i in 0..=53u32 {
        let val = read_gpio_event_detect_status(i);

        if val {
            #[allow(clippy::single_match)]
            match i {
                26 => {
                    println!("Button Pressed");
                }
                _ => {}
            }
            // Reset GPIO Interrupt handler by writing a 1
            reset_gpio_event_detect_status(i);
        }
    }
    unmask_irq();
}

/// Enables IRQ Source
pub fn enable_irq_source(state: IRQSource) {
    let nr = state as u32;
    let register = ENABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    let current = unsafe { read_address(register) };
    let mask = 0b1 << register_offset;
    let new_val = current | mask;
    unsafe { write_address(register, new_val) };
}

/// Disable IRQ Source
pub fn disable_irq_source(state: IRQSource) {
    let nr = state as u32;
    let register = DISABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    let current = unsafe { read_address(register) };
    let mask = 0b1 << register_offset;
    let new_val = current | mask;
    unsafe { write_address(register, new_val) };
}

/// Read current IRQ Source status
pub fn read_irq_source_status(state: IRQSource) -> u32 {
    let nr = state as u32;
    let register = ENABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    (unsafe { read_address(register) } >> register_offset) & 0b1
}

/// Status if a IRQ Source is pending
pub fn is_irq_source_pending(state: IRQSource) -> bool {
    let nr = state as u32;
    let register = IRQ_PENDING_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    ((unsafe { read_address(register) } >> register_offset) & 0b1) != 0
}

/// Status if a IRQ Source is pending
pub fn get_irq_pending_sources() -> u64 {
    let mut pending = unsafe { read_address(IRQ_PENDING_BASE + 4) as u64 } << 32;
    pending |= unsafe { read_address(IRQ_PENDING_BASE) as u64 };
    pending
}

pub mod daif {
    use core::arch::asm;

    #[inline(always)]
    pub fn mask_all() {
        unsafe { asm!("msr DAIFSet, #0xf", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn unmask_all() {
        unsafe { asm!("msr DAIFClr, #0xf", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn mask_irq() {
        unsafe { asm!("msr DAIFSet, #0x2", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn unmask_irq() {
        unsafe { asm!("msr DAIFClr, #0x2", options(nomem, nostack)) }
    }
}
