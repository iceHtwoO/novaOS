use crate::uart::{self};

const GPFSEL_BASE: u32 = 0x3F20_0000;
const GPSET_BASE: u32 = 0x3F20_001C;
const GPCLR_BASE: u32 = 0x3F20_0028;

#[repr(u32)]
pub enum GPIOState {
    input = 0b000,
    output = 0b001,
    alternative0 = 0b100,
    alternative1 = 0b101,
    alternative2 = 0b110,
    alternative3 = 0b111,
    alternative4 = 0b011,
    alternative5 = 0b010,
}

pub unsafe fn set_gpio_state(gpio: u8, state: GPIOState) -> Result<(), &'static str> {
    if gpio > 53 {
        return Err("GPIO out of range");
    }

    let register_index = gpio / 10;
    let register_offset = (gpio % 10) * 3;
    let register_addr = GPFSEL_BASE + (register_index as u32 * 4);

    let current = core::ptr::read_volatile(register_addr as *const u32);

    let mask = !(0b111 << register_offset);
    let cleared = current & mask;

    let new_val = cleared | ((state as u32) << register_offset);

    core::ptr::write_volatile(register_addr as *mut u32, new_val);
    Ok(())
}

pub fn gpio_high(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        uart::print("Pull Up\n");

        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPSET_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

pub fn gpio_low(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        uart::print("Pull Down\n");

        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPCLR_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}
