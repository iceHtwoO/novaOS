use core::arch::asm;

use alloc::string::String;

use crate::{
    interrupt_handlers::irq::{register_interrupt_handler, IRQSource},
    peripherals::uart::read_uart_data,
    pi3::mailbox::read_soc_temp,
    print, println,
};

pub static mut TERMINAL: Option<Terminal> = None;

pub struct Terminal {
    input: String,
}

extern "C" {
    fn el1_to_el0();
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    fn flush(&mut self) {
        print!("\n> {}", self.input);
    }

    fn exec(&mut self) {
        print!("\n");
        let val = self.input.clone();
        self.input.clear();

        match val.as_str() {
            "temp" => {
                println!("{}", read_soc_temp([0]).unwrap()[1]);
            }
            "el0" => unsafe {
                let i = 69;
                asm!("", in("x0") i);
                el1_to_el0();
            },
            _ => {
                println!("Unknown command: \"{}\"", self.input);
            }
        }
        self.input.clear();
    }
}

pub fn init_terminal() {
    unsafe { TERMINAL = Some(Terminal::new()) };
    register_terminal_interrupt_handler();
}

fn terminal_uart_rx_interrupt_handler() {
    let input = read_uart_data();
    #[allow(static_mut_refs)]
    if let Some(term) = unsafe { TERMINAL.as_mut() } {
        match input {
            '\r' => {
                term.exec();
                term.flush();
            }
            _ => {
                term.input.push(input);
                print!("{}", input);
            }
        }
    }
}

pub fn flush_terminal() {
    #[allow(static_mut_refs)]
    if let Some(term) = unsafe { TERMINAL.as_mut() } {
        term.flush();
    }
}

fn register_terminal_interrupt_handler() {
    register_interrupt_handler(IRQSource::UartInt, terminal_uart_rx_interrupt_handler);
}
