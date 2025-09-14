#![no_main]
#![no_std]
#![feature(asm_experimental_arch)]
#![allow(static_mut_refs)]
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    ptr::write_volatile,
};

extern crate alloc;

use nova::{
    framebuffer::{FrameBuffer, BLUE, GREEN, RED},
    heap::{init_global_heap, HEAP},
    irq_interrupt::enable_irq_source,
    mailbox::mb_read_soc_temp,
    peripherals::{
        gpio::{
            blink_gpio, gpio_pull_up, set_falling_edge_detect, set_gpio_function, GPIOFunction,
            SpecificGpio,
        },
        uart::uart_init,
    },
    print, println,
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
        println!("Panic");
    }
}

#[no_mangle]
#[link_section = ".text._start"]
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

    // Delay so clock speed can stabilize
    delay_nops(50000);
    println!("Hello World!");

    unsafe {
        asm!("mrs x0, SCTLR_EL1");
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
    println!("EL: {}", get_current_el());

    heap_test();

    sleep_us(500_000);

    // Set GPIO 26 to Input
    enable_irq_source(nova::irq_interrupt::IRQState::GpioInt0); //26 is on the first GPIO bank
    let _ = set_gpio_function(26, GPIOFunction::Input);
    gpio_pull_up(26);
    set_falling_edge_detect(26, true);

    let fb = FrameBuffer::new();

    fb.draw_square(500, 500, 600, 700, RED);
    fb.draw_square_fill(800, 800, 900, 900, GREEN);
    fb.draw_square_fill(1000, 800, 1200, 700, BLUE);
    fb.draw_square_fill(900, 100, 800, 150, RED | BLUE);
    fb.draw_string("Hello World! :D\nTest next Line", 500, 5, 3, BLUE);

    fb.draw_function(cos, 100, 101, RED);

    loop {
        let temp = mb_read_soc_temp([0]).unwrap();
        println!("{} °C", temp[1] / 1000);

        blink_gpio(SpecificGpio::OnboardLed as u8, 500);
    }
}

fn heap_test() {
    unsafe {
        init_global_heap();
        let a = HEAP.malloc(32).unwrap();
        let b = HEAP.malloc(64).unwrap();
        let c = HEAP.malloc(128).unwrap();
        let _ = HEAP.malloc(256).unwrap();
        HEAP.traverse_heap();
        HEAP.free(b).unwrap();
        HEAP.traverse_heap();
        HEAP.free(a).unwrap();
        HEAP.traverse_heap();
        HEAP.free(c).unwrap();
        HEAP.traverse_heap();
    }
}

fn cos(x: u32) -> f64 {
    libm::cos(x as f64 * 0.1) * 20.0
}

fn get_current_el() -> u64 {
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
