#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::panic::PanicInfo;

use gpio::{pull_down_gpio47, pull_up_gpio47};

mod gpio;
mod uart;

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        uart::print("Panic");
    }
}

#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!("mov sp, #0x80000", "bl main");
}

#[no_mangle]
fn main() {
    uart::configure_uart();

    // Delay so clock speed can stabilize
    unsafe { delay(50000) }
    uart::print("Hello World!\n");

    loop {
        pull_up_gpio47();
        unsafe { delay(10_000_000) } // ~0.5s
        pull_down_gpio47();
        unsafe { delay(10_000_000) }
    }
}

unsafe fn delay(count: u32) {
    for _ in 0..count {
        // Prevent compiler optimizing away the loop
        core::arch::asm!("nop");
    }
}
