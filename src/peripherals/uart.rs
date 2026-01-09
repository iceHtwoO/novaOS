use core::{
    arch::asm,
    fmt::{self, Write},
};

use crate::{println, read_address, write_address};

const BAUD: u32 = 115200;
const UART_CLK: u32 = 48_000_000;

const UART0_DR: u32 = 0x3F20_1000;

const UART0_FR: u32 = 0x3F20_1018;
const UART0_FR_TXFF: u32 = 1 << 5;

const UART0_IBRD: u32 = 0x3F20_1024;
const UART0_FBRD: u32 = 0x3F20_1028;

const UART0_CR: u32 = 0x3F20_1030;
const UART0_CR_UARTEN: u32 = 1 << 0;

const UART0_CR_LBE: u32 = 1 << 7;
const UART0_CR_TXE: u32 = 1 << 8;
const UART0_CR_RXE: u32 = 1 << 9;

const UART0_LCRH: u32 = 0x3F20_102C;
const UART0_LCRH_FEN: u32 = 1 << 4;

/// Initialize UART peripheral
pub fn uart_init() {
    let baud_div_times_64 = (UART_CLK * 4) / BAUD;

    let ibrd = baud_div_times_64 / 64;
    let fbrd = baud_div_times_64 % 64;

    uart_enable(false);
    uart_fifo_enable(true);

    unsafe {
        write_address(UART0_IBRD, ibrd);
        write_address(UART0_FBRD, fbrd);
    }

    uart_set_lcrh(0b11, true);

    // Enable transmit, receive and uart
    let mut cr = unsafe { read_address(UART0_CR) };
    cr |= UART0_CR_UARTEN | UART0_CR_TXE | UART0_CR_RXE;

    unsafe { write_address(UART0_CR, cr) };
}

/// Enable UARTEN
fn uart_enable(enable: bool) {
    let mut cr = unsafe { read_address(UART0_CR) };

    if enable {
        cr |= UART0_CR_UARTEN;
    } else {
        cr &= !UART0_CR_UARTEN;
    }

    unsafe { write_address(UART0_CR, cr) };
}

/// Enable UART FIFO
fn uart_fifo_enable(enable: bool) {
    let mut lcrh = unsafe { read_address(UART0_LCRH) };

    if enable {
        lcrh |= UART0_LCRH_FEN;
    } else {
        lcrh &= !UART0_LCRH_FEN;
    }

    unsafe { write_address(UART0_LCRH, lcrh) };
}

/// Set UART word length and set FIFO status
fn uart_set_lcrh(wlen: u32, enable_fifo: bool) {
    let mut value = (wlen & 0b11) << 5;
    if enable_fifo {
        value |= UART0_LCRH_FEN;
    }
    unsafe { write_address(UART0_LCRH, value) };
}

pub fn read_uart_data() {
    println!(
        "{:?}",
        (unsafe { read_address(UART0_DR) } & 0xFF) as u8 as char
    );
}

pub fn write_str(s: &str) -> core::fmt::Result {
    for byte in s.bytes() {
        while (unsafe { read_address(UART0_FR) } & UART0_FR_TXFF) != 0 {
            unsafe { asm!("nop") }
        }
        unsafe { write_address(UART0_DR, byte as u32) };
    }
    // wait till uart is not busy anymore
    while ((unsafe { read_address(UART0_FR) } >> 3) & 0b1) != 0 {}
    Ok(())
}
