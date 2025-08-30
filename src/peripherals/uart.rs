use core::{
    arch::asm,
    fmt::{self, Write},
};

use crate::{mmio_read, mmio_write};

const BAUD: u32 = 115200;
const UART_CLK: u32 = 48_000_000;

const UART0_DR: u32 = 0x3F20_1000;

const UART0_FR: u32 = 0x3F20_1018;
const UART0_FR_TXFF: u32 = 1 << 5;

const UART0_IBRD: u32 = 0x3F20_1024;
const UART0_FBRD: u32 = 0x3F20_1028;

const UART0_CR: u32 = 0x3F20_1030;
const UART0_CR_UARTEN: u32 = 1 << 0;
const UART0_CR_TXE: u32 = 1 << 8;

const UART0_LCRH: u32 = 0x3F20_102c;
const UART0_LCRH_FEN: u32 = 1 << 4;

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            while (mmio_read(UART0_FR) & UART0_FR_TXFF) != 0 {
                unsafe { asm!("nop") }
            }
            mmio_write(UART0_DR, byte as u32);
        }
        // wait till uart is not busy anymore
        while ((mmio_read(UART0_FR) >> 3) & 0b1) != 0 {}
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let _ = Uart.write_fmt(args);
}

pub fn _print_str(st: &str) {
    let _ = Uart.write_str(st);
}

/// Initialize UART peripheral
pub fn uart_init() {
    let baud_div_times_64 = (UART_CLK * 4) / BAUD;

    let ibrd = baud_div_times_64 / 64;
    let fbrd = baud_div_times_64 % 64;

    uart_enable(false);
    uart_fifo_enable(false);

    mmio_write(UART0_IBRD, ibrd);
    mmio_write(UART0_FBRD, fbrd);

    uart_set_lcrh(0b11, true);

    // Enable transmit and uart
    let mut cr = mmio_read(UART0_CR);
    cr |= UART0_CR_UARTEN | UART0_CR_TXE;

    mmio_write(UART0_CR, cr);
}

/// Enable UARTEN
fn uart_enable(enable: bool) {
    let mut cr = mmio_read(UART0_CR);

    if enable {
        cr |= UART0_CR_UARTEN;
    } else {
        cr &= !UART0_CR_UARTEN;
    }

    mmio_write(UART0_CR, cr);
}

/// Enable UART FIFO
fn uart_fifo_enable(enable: bool) {
    let mut lcrh = mmio_read(UART0_LCRH);

    if enable {
        lcrh |= UART0_LCRH_FEN;
    } else {
        lcrh &= !UART0_LCRH_FEN;
    }

    mmio_write(UART0_LCRH, lcrh);
}

/// Set UART word length and set FIFO status
fn uart_set_lcrh(wlen: u32, enable_fifo: bool) {
    let mut value = (wlen & 0b11) << 5;
    if enable_fifo {
        value |= UART0_LCRH_FEN;
    }
    mmio_write(UART0_LCRH, value);
}
