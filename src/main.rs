#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

use gpio::{
    gpio_enable_low_detect, gpio_get_state, gpio_high, gpio_low, gpio_pull_up, set_gpio_state,
};
use interrupt::enable_iqr_source;
use timer::{delay_nops, sleep};
use uart::print;

mod gpio;
mod interrupt;
mod timer;
mod uart;

global_asm!(include_str!("vector.S"));

extern "C" {
    fn el2_to_el1();
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        uart::print("Panic\r\n");
    }
}

#[no_mangle]
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start() {
    // Set the stack pointer
    asm!(
        "ldr x0, =0x8008000",
        "mov sp, x0",
        "b main",
        options(noreturn)
    );
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    uart::uart_init();
    // Set ACT Led to Outout
    let _ = set_gpio_state(21, gpio::GPIOState::Output);

    // Set GPIO Pins to UART
    let _ = set_gpio_state(14, gpio::GPIOState::Alternative0);
    let _ = set_gpio_state(15, gpio::GPIOState::Alternative0);

    print_current_el_str();

    // Delay so clock speed can stabilize
    delay_nops(50000);
    uart::print("Hello World!\r\n");

    unsafe {
        el2_to_el1();
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let el = get_current_el();
    print_current_el_str();

    sleep(500_000);

    // Set GPIO 21 to Input
    enable_iqr_source(49); //21 is on the first GPIO bank
    let _ = set_gpio_state(21, gpio::GPIOState::Input);
    gpio_pull_up(21);
    gpio_enable_low_detect(21, true);

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

pub fn get_current_el() -> u64 {
    let el: u64;
    unsafe {
        asm!(
            "mrs {el}, CurrentEL",
            el = out(reg) el,
            options(nomem, nostack, preserves_flags)
        );
    }
    el >> 2
}

fn print_current_el_str() {
    let el = get_current_el();
    let el_str = match el {
        0b11 => "Level 3",
        0b10 => "Level 2",
        0b01 => "Level 1",
        0b00 => "Level 0",
        _ => "Unknown EL",
    };

    print(el_str);
    print("\r\n");
}
