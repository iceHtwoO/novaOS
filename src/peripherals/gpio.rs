use core::ptr::{read_volatile, write_volatile};
use core::result::Result;
use core::result::Result::Ok;

use crate::timer::delay_nops;

const GPFSEL_BASE: u32 = 0x3F20_0000;
const GPSET_BASE: u32 = 0x3F20_001C;
const GPCLR_BASE: u32 = 0x3F20_0028;
const GPLEV_BASE: u32 = 0x3F20_0034;
const GPPUD: u32 = 0x3F20_0094;
const GPPUDCLK_BASE: u32 = 0x3F20_0098;
const GPREN_BASE: u32 = 0x3F20_004C;
const GPFEN_BASE: u32 = 0x3F20_0058;

#[repr(u32)]
pub enum GPIOFunction {
    Input = 0b000,
    Output = 0b001,
    Alternative0 = 0b100,
    Alternative1 = 0b101,
    Alternative2 = 0b110,
    Alternative3 = 0b111,
    Alternative4 = 0b011,
    Alternative5 = 0b010,
}

/// Set the function of the GPIO pin
pub fn set_gpio_function(gpio: u8, state: GPIOFunction) -> Result<(), &'static str> {
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

/// Set the GPIO to high
///
/// Should be used when GPIO function is set to `OUTPUT` via `set_gpio_function`
pub fn gpio_high(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPSET_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

/// Set the GPIO to low
///
/// Should be used when GPIO function is set to `OUTPUT` via `set_gpio_function`
pub fn gpio_low(gpio: u8) -> Result<(), &'static str> {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPCLR_BASE + (register_index as u32 * 4);

        core::ptr::write_volatile(register_addr as *mut u32, 1 << register_offset);
    }
    Ok(())
}

/// Read the current GPIO power state
pub fn gpio_get_state(gpio: u8) -> u8 {
    unsafe {
        let register_index = gpio / 32;
        let register_offset = gpio % 32;
        let register_addr = GPLEV_BASE + (register_index as u32 * 4);

        let state = core::ptr::read_volatile(register_addr as *mut u32);
        return ((state >> register_offset) & 0b1) as u8;
    }
}

/// Pull GPIO up
///
/// Should be used when GPIO function is set to `INPUT` via `set_gpio_function`
pub fn gpio_pull_up(gpio: u8) {
    gpio_pull_up_down(gpio, 0b10);
}

/// Pull GPIO down
///
/// Should be used when GPIO function is set to `INPUT` via `set_gpio_function`
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

/// Get the current status if falling edge detection is set
pub fn read_falling_edge_detect(gpio: u8) -> bool {
    unsafe {
        let register_addr = GPFEN_BASE + 4 * (gpio as u32 / 32);
        let register_offset = gpio % 32;

        let current = read_volatile(register_addr as *const u32);
        ((current >> register_offset) & 0b1) != 0
    }
}

/// Get the current status if falling edge detection is set
pub fn read_rising_edge_detect(gpio: u8) -> bool {
    unsafe {
        let register_addr = GPREN_BASE + 4 * (gpio as u32 / 32);
        let register_offset = gpio % 32;

        let current = read_volatile(register_addr as *const u32);
        ((current >> register_offset) & 0b1) != 0
    }
}

/// Enables falling edge detection
pub fn set_falling_edge_detect(gpio: u8, enable: bool) {
    unsafe {
        let register_addr = GPFEN_BASE + 4 * (gpio as u32 / 32);
        let register_offset = gpio % 32;

        let current = read_volatile(register_addr as *const u32);
        let mask = 0b1 << register_offset;
        let new_val = if enable {
            current | mask
        } else {
            current & !mask
        };

        write_volatile(register_addr as *mut u32, new_val);
    }
}

/// Enables rising edge detection
pub fn set_rising_edge_detect(gpio: u8, enable: bool) {
    unsafe {
        let register_addr = GPREN_BASE + 4 * (gpio as u32 / 32);
        let register_offset = gpio % 32;

        let current = read_volatile(register_addr as *const u32);

        let mask = 0b1 << register_offset;
        let new_val = if enable {
            current | mask
        } else {
            current & !mask
        };

        write_volatile(register_addr as *mut u32, new_val);
    }
}
