#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::{arch::asm, panic::PanicInfo};

use gpio::{gpio_high, gpio_low, set_gpio_state};

mod gpio;
mod uart;

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        uart::print("Panic");
    }
}

#[no_mangle]
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start() {
    asm!("ldr x0, =0x8004000", "mov sp, x0");
    main();
}

#[no_mangle]
extern "C" fn main() {
    uart::configure_uart();
    unsafe {
        let _ = set_gpio_state(29, gpio::GPIOState::output);
    }

    // Delay so clock speed can stabilize
    unsafe { delay(50000) }
    uart::print("Hello World!\n");

    loop {
        let _ = gpio_high(29);
        unsafe { delay(1_000_000) }
        let _ = gpio_low(29);
        unsafe { delay(1_000_000) }
    }
}

unsafe fn delay(count: u32) {
    for _ in 0..count {
        // Prevent compiler optimizing away the loop
        core::arch::asm!("nop");
    }
}
