use core::{
    arch::asm,
    sync::atomic::{compiler_fence, Ordering},
};

use crate::{mmio_read, mmio_write, peripherals::uart::print};

const INTERRUPT_BASE: u32 = 0x3F00_B000;
const IRQ_PENDING_BASE: u32 = INTERRUPT_BASE + 0x204;
const ENABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x210;
const DISABLE_IRQ_BASE: u32 = INTERRUPT_BASE + 0x21C;

// GPIO
const GPEDS_BASE: u32 = 0x3F20_0040;

#[repr(u32)]
pub enum IRQState {
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

#[no_mangle]
unsafe extern "C" fn irq_handler() {
    handle_gpio_interrupt();
}

fn handle_gpio_interrupt() {
    for i in 0..=53u32 {
        let val = read_gpio_event_detect_status(i);

        if val {
            match i {
                26 => print("Button Pressed"),
                _ => {}
            }
            // Reset GPIO Interrupt handler by writing a 1
            reset_gpio_event_detect_status(i);
        }
    }
    enable_irq();
}

/// Get current interrupt status of a GPIO pin
pub fn read_gpio_event_detect_status(id: u32) -> bool {
    let register = GPEDS_BASE + (id / 32) * 4;
    let register_offset = id % 32;

    let val = mmio_read(register) >> register_offset;
    (val & 0b1) != 0
}

/// Resets current interrupt status of a GPIO pin
pub fn reset_gpio_event_detect_status(id: u32) {
    let register = GPEDS_BASE + (id / 32) * 4;
    let register_offset = id % 32;

    mmio_write(register, 0b1 << register_offset);
    compiler_fence(Ordering::SeqCst);
}

/// Enables IRQ Source
pub fn enable_irq_source(state: IRQState) {
    let nr = state as u32;
    let register = ENABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    let current = mmio_read(register);
    let mask = 0b1 << register_offset;
    let new_val = current | mask;
    mmio_write(register, new_val);
}

/// Disable IRQ Source
pub fn disable_irq_source(state: IRQState) {
    let nr = state as u32;
    let register = DISABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    let current = mmio_read(register);
    let mask = 0b1 << register_offset;
    let new_val = current | mask;
    mmio_write(register, new_val);
}

/// Read current IRQ Source status
pub fn read_irq_source_status(state: IRQState) -> u32 {
    let nr = state as u32;
    let register = ENABLE_IRQ_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    (mmio_read(register) >> register_offset) & 0b1
}

/// Status if a IRQ Source is enabled
pub fn read_irq_pending(state: IRQState) -> bool {
    let nr = state as u32;
    let register = IRQ_PENDING_BASE + 4 * (nr / 32);
    let register_offset = nr % 32;
    ((mmio_read(register) >> register_offset) & 0b1) != 0
}

/// Clears the IRQ DAIF Mask
///
/// Enables IRQ interrupts
pub fn enable_irq() {
    unsafe { asm!("msr DAIFClr, #0x2") }
}

/// Clears the IRQ DAIF Mask
///
/// Disable IRQ interrupts
pub fn disable_irq() {
    unsafe { asm!("msr DAIFSet, #0x2") }
}
