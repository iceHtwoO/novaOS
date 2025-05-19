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

pub fn print(s: &str) {
    for byte in s.bytes() {
        unsafe {
            while core::ptr::read_volatile(UART0_FR as *const u32) & UART0_FR_TXFF != 0 {}
            core::ptr::write_volatile(UART0_DR as *mut u32, byte as u32);
        }
    }
}

pub fn configure_uart() {
    let baud_div_times_64 = (UART_CLK * 4) / BAUD;

    let ibrd = baud_div_times_64 / 64;
    let fbrd = baud_div_times_64 % 64;

    unsafe {
        uart_enable(false);
        uart_fifo_enable(false);

        core::ptr::write_volatile(UART0_IBRD as *mut u32, ibrd);
        core::ptr::write_volatile(UART0_FBRD as *mut u32, fbrd);

        uart_set_lcrh(0b11, true);

        // Enable transmit and uart
        let mut cr = core::ptr::read_volatile(UART0_CR as *mut u32);
        cr |= UART0_CR_UARTEN | UART0_CR_TXE;
        core::ptr::write_volatile(UART0_CR as *mut u32, cr);
    }
}

fn uart_enable(enable: bool) {
    unsafe {
        let mut cr = core::ptr::read_volatile(UART0_CR as *mut u32);

        if enable {
            cr |= UART0_CR_UARTEN;
        } else {
            cr &= !UART0_CR_UARTEN;
        }

        core::ptr::write_volatile(UART0_CR as *mut u32, cr);
    }
}

fn uart_fifo_enable(enable: bool) {
    unsafe {
        let mut lcrh = core::ptr::read_volatile(UART0_LCRH as *mut u32);

        if enable {
            lcrh |= UART0_LCRH_FEN;
        } else {
            lcrh &= !UART0_LCRH_FEN;
        }

        core::ptr::write_volatile(UART0_LCRH as *mut u32, lcrh);
    }
}

fn uart_set_lcrh(wlen: u32, enable_fifo: bool) {
    unsafe {
        let mut value = (wlen & 0b11) << 5;
        if enable_fifo {
            value |= UART0_LCRH_FEN;
        }
        core::ptr::write_volatile(UART0_LCRH as *mut u32, value);
    }
}
