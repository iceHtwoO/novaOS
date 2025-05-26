#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::{arch::asm, panic::PanicInfo};

use gpio::{gpio_get_state, gpio_high, gpio_low, gpio_pull_up, set_gpio_state};
use timer::{delay_nops, sleep};
use uart::print;

mod gpio;
mod timer;
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
    // Set the stack pointer
    asm!("ldr x0, =0x8004000", "mov sp, x0");
    main();
}

#[no_mangle]
extern "C" fn main() {
    uart::configure_uart();
    // Set ACT Led to Outout
    let _ = set_gpio_state(21, gpio::GPIOState::Output);

    // Set GPIO Pins to UART
    let _ = set_gpio_state(14, gpio::GPIOState::Alternative0);
    let _ = set_gpio_state(15, gpio::GPIOState::Alternative0);

    // Set GPIO 21 to Input
    let _ = set_gpio_state(21, gpio::GPIOState::Input);
    gpio_pull_up(21);

    // Delay so clock speed can stabilize
    delay_nops(50000);
    uart::print("Hello World!\r\n");

    sleep(500_000);

    loop {
        let _ = gpio_high(29);

        sleep(500_000); // 0.5s
        let _ = gpio_low(29);
        sleep(500_000); // 0.5s
        print_gpio_state();
    }
}

fn print_gpio_state() {
    let state = gpio_get_state(21);

    let ascii_byte = b'0' + state;
    let data = [ascii_byte];

    let s = str::from_utf8(&data).unwrap();
    print(s);
    print("\r\n");
}
