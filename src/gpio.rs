use core::ptr::{read_volatile, write_volatile};

use crate::{
    timer::delay_nops,
    uart::{self},
};

const GPFSEL_BASE: u32 = 0x3F20_0000;
const GPSET_BASE: u32 = 0x3F20_001C;
const GPCLR_BASE: u32 = 0x3F20_0028;
const GPLEV_BASE: u32 = 0x3F20_0034;
const GPPUD: u32 = 0x3F20_0094;
const GPPUDCLK_BASE: u32 = 0x3F20_0098;

#[repr(u32)]
pub enum GPIOState {
    Input = 0b000,
    Output = 0b001,
    Alternative0 = 0b100,
    Alternative1 = 0b101,
    Alternative2 = 0b110,
    Alternative3 = 0b111,
    Alternative4 = 0b011,
    Alternative5 = 0b010,
}

pub fn set_gpio_state(gpio: u8, state: GPIOState) -> Result<(), &'static str> {
    if gpio > 53 {
        return Err("GPIO out of range");
    }

    let register_index = gpio / 10;
    let register_offset = (gpio % 10) * 3;
    let register_addr = GPFSEL_BASE + (register_index as u32 * 4);
    unsafe {
        let current = core::ptr::read_volatile(register_addr as *const u32);

        let mask = !(0b111 << register_offset);
        let cleared = current & mask;

        let new_val = cleared | ((state as u32) << register_offset);

        core::ptr::write_volatile(register_addr as *mut u32, new_val);
    }
    Ok(())
}

pub fn gpio_high(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPSET_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

pub fn gpio_low(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPCLR_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

pub fn gpio_get_state(gpio: u8) -> u8 {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPLEV_BASE + (register_index as u32 * 4);

        let state = core::ptr::read_volatile(register_addr as *mut u32);
        return ((state >> register_offset) & 0b1) as u8;
    }
}

pub fn gpio_pull_up(gpio: u8) {
    gpio_pull_up_down(gpio, 0b10);
}

pub fn gpio_pull_down(gpio: u8) {
    gpio_pull_up_down(gpio, 0b01);
}

fn gpio_pull_up_down(gpio: u8, val: u32) {
    unsafe {
        // Determine GPPUDCLK Register
        let register_addr = GPPUDCLK_BASE + 4 * (gpio as u32 / 32);
        let register_offset = gpio % 32;

        // 1. Write Pull up
        write_volatile(GPPUD as *mut u32, val);

        // 2. Delay 150 cycles
        delay_nops(150);

        // 3. Write to clock
        let new_val = 0b1 << register_offset;
        write_volatile(register_addr as *mut u32, new_val);

        // 4. Delay 150 cycles
        delay_nops(150);

        // 5. reset GPPUD
        write_volatile(GPPUD as *mut u32, 0);

        // 6. reset clock
        write_volatile(register_addr as *mut u32, 0);
    }
}
