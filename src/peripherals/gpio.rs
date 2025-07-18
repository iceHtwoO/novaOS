use core::result::Result;
use core::result::Result::Ok;

use crate::timer::{delay_nops, sleep_ms};
use crate::{mmio_read, mmio_write};

const GPFSEL_BASE: u32 = 0x3F20_0000;
const GPSET_BASE: u32 = 0x3F20_001C;
const GPCLR_BASE: u32 = 0x3F20_0028;
const GPLEV_BASE: u32 = 0x3F20_0034;
const GPPUD: u32 = 0x3F20_0094;
const GPPUDCLK_BASE: u32 = 0x3F20_0098;
const GPREN_BASE: u32 = 0x3F20_004C;
const GPFEN_BASE: u32 = 0x3F20_0058;

#[repr(u8)]
pub enum SpecificGpio {
    OnboardLed = 29,
}

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
    let current = mmio_read(register_addr);

    let mask = !(0b111 << register_offset);
    let cleared = current & mask;

    let new_val = cleared | ((state as u32) << register_offset);

    mmio_write(register_addr, new_val);
    Ok(())
}

/// Set the GPIO to high
///
/// Should be used when GPIO function is set to `OUTPUT` via `set_gpio_function`
pub fn gpio_high(gpio: u8) -> Result<(), &'static str> {
    let register_index = gpio / 32;
    let register_offset = gpio % 32;
    let register_addr = GPSET_BASE + (register_index as u32 * 4);

    mmio_write(register_addr, 1 << register_offset);
    Ok(())
}

/// Set the GPIO to low
///
/// Should be used when GPIO function is set to `OUTPUT` via `set_gpio_function`
pub fn gpio_low(gpio: u8) -> Result<(), &'static str> {
    let register_index = gpio / 32;
    let register_offset = gpio % 32;
    let register_addr = GPCLR_BASE + (register_index as u32 * 4);

    mmio_write(register_addr, 1 << register_offset);
    Ok(())
}

/// Read the current GPIO power state
pub fn gpio_get_state(gpio: u8) -> u8 {
    let register_index = gpio / 32;
    let register_offset = gpio % 32;
    let register_addr = GPLEV_BASE + (register_index as u32 * 4);

    let state = mmio_read(register_addr);
    return ((state >> register_offset) & 0b1) as u8;
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
    // Determine GPPUDCLK Register
    let register_addr = GPPUDCLK_BASE + 4 * (gpio as u32 / 32);
    let register_offset = gpio % 32;

    // 1. Write Pull up
    mmio_write(GPPUD, val);

    // 2. Delay 150 cycles
    delay_nops(150);

    // 3. Write to clock
    let new_val = 0b1 << register_offset;
    mmio_write(register_addr, new_val);

    // 4. Delay 150 cycles
    delay_nops(150);

    // 5. reset GPPUD
    mmio_write(GPPUD, 0);

    // 6. reset clock
    mmio_write(register_addr, 0);
}

/// Get the current status if falling edge detection is set
pub fn read_falling_edge_detect(gpio: u8) -> bool {
    let register_addr = GPFEN_BASE + 4 * (gpio as u32 / 32);
    let register_offset = gpio % 32;

    let current = mmio_read(register_addr);
    ((current >> register_offset) & 0b1) != 0
}

/// Get the current status if falling edge detection is set
pub fn read_rising_edge_detect(gpio: u8) -> bool {
    let register_addr = GPREN_BASE + 4 * (gpio as u32 / 32);
    let register_offset = gpio % 32;

    let current = mmio_read(register_addr);
    ((current >> register_offset) & 0b1) != 0
}

/// Enables falling edge detection
pub fn set_falling_edge_detect(gpio: u8, enable: bool) {
    let register_addr = GPFEN_BASE + 4 * (gpio as u32 / 32);
    let register_offset = gpio % 32;

    let current = mmio_read(register_addr);
    let mask = 0b1 << register_offset;
    let new_val = if enable {
        current | mask
    } else {
        current & !mask
    };

    mmio_write(register_addr, new_val);
}

/// Enables rising edge detection
pub fn set_rising_edge_detect(gpio: u8, enable: bool) {
    let register_addr = GPREN_BASE + 4 * (gpio as u32 / 32);
    let register_offset = gpio % 32;

    let current = mmio_read(register_addr);

    let mask = 0b1 << register_offset;
    let new_val = if enable {
        current | mask
    } else {
        current & !mask
    };

    mmio_write(register_addr, new_val);
}

pub fn blink_gpio(gpio: u8, duration_ms: u32) {
    let _ = gpio_high(gpio);

    sleep_ms(duration_ms);
    let _ = gpio_low(gpio);
    sleep_ms(duration_ms);
}
