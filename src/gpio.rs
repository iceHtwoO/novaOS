use crate::{
    delay,
    uart::{self},
};

const GPFSEL4: u32 = 0x3F20_0010; //GPIO 40-49
const GPPUD: u32 = 0x3F20_0094;
const GPPUD_CLK: u32 = 0x3F20_0098; // GPIO 32-53

unsafe fn set_gpio47_to_output() {
    let value: u32 = 0b001 << 21;

    core::ptr::write_volatile(GPFSEL4 as *mut u32, value);
}

unsafe fn enable_pull_up() {
    let value: u32 = 0b10;

    core::ptr::write_volatile(GPPUD as *mut u32, value);
}

unsafe fn enable_pull_down() {
    let value: u32 = 0b01;

    core::ptr::write_volatile(GPPUD as *mut u32, value);
}

unsafe fn disable_GPPUD() {
    core::ptr::write_volatile(GPPUD as *mut u32, 0);
}

unsafe fn enable_clock_gpio47() {
    let value: u32 = 0b1 << 13;

    core::ptr::write_volatile(GPPUD as *mut u32, value);
}

unsafe fn disable_GPPUD_CLK() {
    core::ptr::write_volatile(GPPUD as *mut u32, 0);
}

pub fn pull_up_gpio47() {
    unsafe {
        uart::print("Pull Up\n");
        set_gpio47_to_output();
        enable_pull_up();
        // Wait 150 cycles
        delay(150);
        enable_clock_gpio47();
        delay(150);
        disable_GPPUD();
        disable_GPPUD_CLK();
    }
}

pub fn pull_down_gpio47() {
    unsafe {
        uart::print("Pull Down\n");
        set_gpio47_to_output();
        enable_pull_down();
        // Wait 150 cycles
        delay(150);
        enable_clock_gpio47();
        delay(150);
        disable_GPPUD();
        disable_GPPUD_CLK();
    }
}
