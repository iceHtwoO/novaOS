use core::ptr::read_volatile;

use crate::{mmio_read, mmio_write, peripherals::uart::print};

const MBOX_BASE: u32 = 0x3F00_0000 + 0xB880;

// MB0
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_STATUS: u32 = MBOX_BASE + 0x18;

// MB1
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;

// Status
const MAIL_FULL: u32 = 0x80000000;
const MAIL_EMPTY: u32 = 0x40000000;

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
    let mut mailbox = [0; 36];
    mailbox[0] = 8 * 4; // Total size in bytes
    mailbox[1] = 0; // Request
    mailbox[2] = 0x00030006; // Tag
    mailbox[3] = 8; // Maximum buffer len
    mailbox[4] = 4; // Request length
    mailbox[5] = 0; // Value Buffer
    mailbox[6] = 0; // Value Buffer
    mailbox[7] = 0; // End

    let addr = core::ptr::addr_of!(mailbox[0]) as u32;

    write_mailbox(8, addr);

    let _ = read_mailbox(8);

    if mailbox[1] == 0 {
        print("Failed\r\n");
    }
    let raw_temp = mailbox[6] / 1000;
    raw_temp
}
