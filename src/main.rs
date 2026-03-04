#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]
#![allow(static_mut_refs)]
#![allow(clippy::missing_safety_doc)]
use core::{
    arch::{asm, global_asm},
    ptr::write_volatile,
};

extern crate alloc;

use alloc::boxed::Box;
use nova::{
    aarch64::registers::{daif, read_id_aa64mmfr0_el1, read_tcr_el1},
    framebuffer::{FrameBuffer, BLUE, GREEN, RED},
    get_current_el, init_heap,
    interrupt_handlers::{enable_irq_source, IRQSource},
    log,
    peripherals::{
        gpio::{
            blink_gpio, gpio_pull_up, set_falling_edge_detect, set_gpio_function, GPIOFunction,
            SpecificGpio,
        },
        uart::uart_init,
    },
    pi3::mailbox,
    println,
};

global_asm!(include_str!("vector.S"));

extern "C" {
    fn el2_to_el1();
    fn el1_to_el0();
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

#[no_mangle]
#[cfg_attr(not(test), link_section = ".text._start")]
pub unsafe extern "C" fn _start() {
    // Set the stack pointer
    asm!(
        "ldr x0, =__stack_end",
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

    println!("Hello World!");
    println!("Exception level: {}", get_current_el());

    unsafe {
        asm!("mrs x0, SCTLR_EL1",);
        el2_to_el1();
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

unsafe fn zero_bss() {
    let mut bss: *mut u32 = &raw mut __bss_start;
    while bss < &raw mut __bss_end {
        write_volatile(bss, 0);
        bss = bss.add(1);
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    nova::initialize_kernel();
    println!("Kernel Main");
    println!("Exception Level: {}", get_current_el());
    daif::unmask_all();

    unsafe {
        init_heap();
        println!("{:b}", read_id_aa64mmfr0_el1());
        println!("{:b}", read_tcr_el1());
        el1_to_el0();
    };

    #[allow(clippy::empty_loop)]
    loop {}
}

#[no_mangle]
pub extern "C" fn el0() -> ! {
    println!("Jumped into EL0");

    // Set GPIO 26 to Input
    enable_irq_source(IRQSource::GpioInt0); //26 is on the first GPIO bank
    let _ = set_gpio_function(26, GPIOFunction::Input);
    gpio_pull_up(26);
    set_falling_edge_detect(26, true);

    enable_irq_source(IRQSource::UartInt);

    let fb = FrameBuffer::default();

    fb.draw_square(500, 500, 600, 700, RED);
    fb.draw_square_fill(800, 800, 900, 900, GREEN);
    fb.draw_square_fill(1000, 800, 1200, 700, BLUE);
    fb.draw_square_fill(900, 100, 800, 150, RED | BLUE);
    fb.draw_string("Hello World! :D\nTest next Line", 500, 5, 3, BLUE);

    fb.draw_function(cos, 100, 101, RED);

    loop {
        let temp = mailbox::read_soc_temp([0]).unwrap();
        log!("{} °C", temp[1] / 1000);

        blink_gpio(SpecificGpio::OnboardLed as u8, 500);

        let b = Box::new([1, 2, 3, 4]);
        log!("{:?}", b);
    }
}

fn cos(x: u32) -> f64 {
    libm::cos(x as f64 * 0.1) * 20.0
}

fn enable_uart() {
    // Set GPIO Pins to UART
    let _ = set_gpio_function(14, GPIOFunction::Alternative0);
    let _ = set_gpio_function(15, GPIOFunction::Alternative0);
    uart_init();
}
