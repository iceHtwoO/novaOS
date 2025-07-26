#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr::write_volatile,
};

use nova::{
    framebuffer::{print_display_resolution, FrameBuffer},
    irq_interrupt::enable_irq_source,
    mailbox::read_soc_temp,
    peripherals::{
        gpio::{
            blink_gpio, gpio_pull_up, set_falling_edge_detect, set_gpio_function, GPIOFunction,
            SpecificGpio,
        },
        uart::{print, print_u32, uart_init},
    },
    timer::{delay_nops, sleep_us},
};

global_asm!(include_str!("vector.S"));

extern "C" {
    fn el2_to_el1();
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        print("Panic\r\n");
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
    unsafe {
        zero_bss();
    }
    enable_uart();

    // Set ACT Led to Outout
    let _ = set_gpio_function(21, GPIOFunction::Output);

    print_current_el_str();

    // Delay so clock speed can stabilize
    delay_nops(50000);
    print("Hello World!\r\n");

    unsafe {
        asm!("mrs x0, SCTLR_EL1");
        el2_to_el1();
    }

    loop {}
}

unsafe fn zero_bss() {
    let mut bss: *mut u32 = &raw mut __bss_start as *mut u32;
    while bss < &raw mut __bss_end as *mut u32 {
        write_volatile(bss, 0);
        bss = bss.add(1);
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    print_current_el_str();

    sleep_us(500_000);

    // Set GPIO 26 to Input
    enable_irq_source(nova::irq_interrupt::IRQState::GpioInt0); //26 is on the first GPIO bank
    let _ = set_gpio_function(26, GPIOFunction::Input);
    gpio_pull_up(26);
    set_falling_edge_detect(26, true);

    print_display_resolution();
    let fb = FrameBuffer::new();
    print_display_resolution();

    fb.draw_line(10, 10, 1000, 10);
    fb.draw_line(10, 10, 1000, 200);
    fb.draw_line(10, 10, 1000, 300);
    fb.draw_line(10, 10, 1000, 400);
    fb.draw_line(10, 10, 1000, 500);
    fb.draw_line(10, 10, 1000, 600);
    fb.draw_line(10, 10, 1000, 700);
    fb.draw_line(10, 10, 1000, 800);
    fb.draw_line(10, 10, 1000, 900);
    fb.draw_line(10, 10, 1000, 1000);
    fb.draw_line(10, 10, 100, 1000);

    fb.draw_line(1800, 10, 1000, 900);
    fb.draw_line(1800, 500, 1000, 100);

    loop {
        let temp = read_soc_temp();
        print_u32(temp);
        print("\r\n");

        blink_gpio(SpecificGpio::OnboardLed as u8, 500);
    }
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

fn enable_uart() {
    uart_init();
    // Set GPIO Pins to UART
    let _ = set_gpio_function(14, GPIOFunction::Alternative0);
    let _ = set_gpio_function(15, GPIOFunction::Alternative0);
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
