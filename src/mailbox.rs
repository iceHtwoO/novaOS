use core::ptr::read_volatile;

use crate::{
    mmio_read, mmio_write,
    peripherals::uart::{print, print_u32},
};

const MBOX_BASE: u32 = 0x3F00_0000 + 0xB880;

// MB0
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_STATUS: u32 = MBOX_BASE + 0x18;

// MB1
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;

// Status
const MAIL_FULL: u32 = 0x80000000;
const MAIL_EMPTY: u32 = 0x40000000;

#[repr(C, align(16))]
struct MailboxBuffer([u32; 36]);

#[no_mangle]
static mut MBOX: MailboxBuffer = MailboxBuffer([0; 36]);

pub fn read_mailbox(channel: u32) -> u32 {
    // Wait until mailbox is not empty
    loop {
        while mmio_read(MBOX_STATUS) & MAIL_EMPTY != 0 {}
        let mut data = mmio_read(MBOX_READ);
        let read_channel = data & 0xF;

        data >>= 4;

        if channel == read_channel {
            return data;
        }
    }
}

pub fn write_mailbox(channel: u32, data: u32) {
    while mmio_read(MBOX_STATUS) & MAIL_FULL != 0 {}
    mmio_write(MBOX_WRITE, (data & !0xF) | (channel & 0xF));
}

pub fn read_soc_temp() -> u32 {
    unsafe {
        // MBOX.0[0] = 7 * 4; // Total size in bytes
        // MBOX.0[1] = 0; // Request
        // MBOX.0[2] = 0x00010002; // Tag
        // MBOX.0[3] = 4; // Maximum buffer lenb
        // MBOX.0[4] = 0; // Request length
        // MBOX.0[5] = 0; // Value Buffer
        // MBOX.0[6] = 0; // End
        // core::arch::asm!("dsb sy"); // Ensure write reaches RAM
        // core::arch::asm!("dmb sy"); // Memory barrier

        print("Reading address\r\n");
        //let addr = core::ptr::addr_of!(MBOX.0[0]);

        print("Write address\r\n");

        // write_mailbox(8, addr);

        let _ = read_mailbox(8);

        if MBOX.0[1] == 0 {
            print("Failed\r\n");
        }
        let raw_temp = MBOX.0[5];
        raw_temp
    }
}
