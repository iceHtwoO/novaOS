use crate::aarch64::registers::read_esr_el1;
use crate::{
    aarch64::registers::{
        daif::{mask_all, unmask_irq},
        read_exception_source_el,
    },
    get_current_el,
    interrupt_handlers::{
        DISABLE_IRQ_BASE, ENABLE_IRQ_BASE, GPIO_PENDING_BIT_OFFSET, IRQ_PENDING_BASE,
    },
    peripherals::{
        gpio::{read_gpio_event_detect_status, reset_gpio_event_detect_status},
        uart::clear_uart_interrupt_state,
    },
    println, read_address, write_address,
};
use alloc::vec::Vec;
use log::{debug, info};

struct InterruptHandlers {
    source: IRQSource,
    function: fn(),
}

// TODO: replace with hashmap and check for better alternatives for option
static mut INTERRUPT_HANDLERS: Option<Vec<InterruptHandlers>> = None;

#[derive(Clone)]
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

#[inline(always)]
pub fn initialize_interrupt_handler() {
    unsafe { INTERRUPT_HANDLERS = Some(Vec::new()) };
}

pub fn register_interrupt_handler(source: IRQSource, function: fn()) {
    if let Some(handler_vec) = unsafe { &mut *core::ptr::addr_of_mut!(INTERRUPT_HANDLERS) } {
        handler_vec.push(InterruptHandlers { source, function });
    }
}

#[no_mangle]
unsafe extern "C" fn rust_irq_handler() {
    mask_all();
    let pending_irqs = get_irq_pending_sources();

    if pending_irqs & GPIO_PENDING_BIT_OFFSET != 0 {
        handle_gpio_interrupt();
        let source_el = read_exception_source_el() >> 2;
        debug!("Source EL: {}", source_el);
        debug!("Current EL: {}", get_current_el());
        debug!("Return register address: {:#x}", read_esr_el1());
    }

    if let Some(handler_vec) = unsafe { &*core::ptr::addr_of_mut!(INTERRUPT_HANDLERS) } {
        for handler in handler_vec {
            if (pending_irqs & (1 << (handler.source.clone() as u32))) != 0 {
                (handler.function)();
                clear_interrupt_for_source(handler.source.clone());
            }
        }
    }
}

fn handle_gpio_interrupt() {
    debug!("GPIO interrupt triggered");
    for i in 0..=53u32 {
        let val = read_gpio_event_detect_status(i);

        if val {
            #[allow(clippy::single_match)]
            match i {
                26 => {
                    info!("Button Pressed");
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

fn clear_interrupt_for_source(source: IRQSource) {
    match source {
        IRQSource::UartInt => clear_uart_interrupt_state(),
        _ => {
            todo!()
        }
    }
}
