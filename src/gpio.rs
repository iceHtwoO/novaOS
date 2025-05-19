use crate::uart::{self};

const GPFSEL_BASE: u32 = 0x3F20_0000;
const GPSET_BASE: u32 = 0x3F20_001C;
const GPCLR_BASE: u32 = 0x3F20_0028;

unsafe fn set_gpio_to_output(gpio: u8) -> Result<(), &'static str> {
    if gpio > 53 {
        return Err("GPIO out of range");
    }

    let register_index = gpio / 10;
    let register_offset = (gpio % 10) * 3;
    let register_addr = GPFSEL_BASE + (register_index as u32 * 4);

    let current = core::ptr::read_volatile(register_addr as *const u32);

    let mask = !(0b111 << register_offset);
    let cleared = current & mask;

    let new_val = cleared | (0b001 << register_offset);

    core::ptr::write_volatile(register_addr as *mut u32, new_val);
    Ok(())
}

pub fn pull_up_gpio(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        uart::print("Pull Up\n");
        set_gpio_to_output(29)?;

        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPSET_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

pub fn pull_down_gpio(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        uart::print("Pull Down\n");

        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPCLR_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}
