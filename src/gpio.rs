use crate::{
    delay,
    uart::{self},
};

const GPFSEL2: u32 = 0x3F20_0008; //GPIO 20-29
const GPFSEL4: u32 = 0x3F20_0010; //GPIO 40-49
const GPSET0: u32 = 0x3F20_001C;
const GPCLR0: u32 = 0x3F20_0028;

unsafe fn set_gpio29_to_output() {
    let value: u32 = 0b001 << 27;

    core::ptr::write_volatile(GPFSEL2 as *mut u32, value);
}

pub fn pull_up_gpio29() {
    unsafe {
        uart::print("Pull Up\n");
        set_gpio29_to_output();
        core::ptr::write_volatile(GPSET0 as *mut u32, 1 << 29);
    }
}

pub fn pull_down_gpio29() {
    unsafe {
        uart::print("Pull Down\n");
        core::ptr::write_volatile(GPCLR0 as *mut u32, 1 << 29);
    }
}
