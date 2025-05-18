#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::{
    arch::asm,
    fmt::{self, Write},
    panic::PanicInfo,
};

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}

const UART0_DR: u32 = 0x3F20_1000;

struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe {
                core::ptr::write_volatile(UART0_DR as *mut u8, byte);
            }
        }
        Ok(())
    }
}

#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!("mov sp, #0x80000", "bl main");
}

#[no_mangle]
fn main() {
    let mut uart = Uart {};
    writeln!(uart, "Hello World!\n");
    loop {}
}
