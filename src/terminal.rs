use core::fmt::Write;

use alloc::string::String;
use nova::{
    interrupt_handlers::register_interrupt_handler, logger::Logger,
    peripherals::uart::read_uart_data, print, println,
};

pub struct Terminal {
    buffer: String,
    input: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            input: String::new(),
        }
    }

    fn flush(&mut self) {
        println!("{}", self.buffer);
        print!("> {}", self.input);
        self.buffer.clear();
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.buffer.push_str(s);
        Ok(())
    }
}

impl Logger for Terminal {
    fn flush(&mut self) {
        println!("{}", self.buffer);
        print!("> {}", self.input);
        self.buffer.clear();
    }
}

fn terminal_uart_rx_interrupt_handler() {
    print!("{}", read_uart_data());
}

pub fn register_terminal_interrupt_handler() {
    register_interrupt_handler(
        nova::interrupt_handlers::IRQSource::UartInt,
        terminal_uart_rx_interrupt_handler,
    );
}
