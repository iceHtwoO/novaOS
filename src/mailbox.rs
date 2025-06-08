use crate::{mmio_read, mmio_write};

const MBOX_BASE: u32 = 0x3F00_B880;

// MB0
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_READ_STATUS: u32 = MBOX_BASE + 0x18;

// MB1
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;
const MBOX_WRITE_STATUS: u32 = MBOX_BASE + 0x38;

// Status
const MAIL_FULL: u32 = 0x80000000;
const MAIL_EMPTY: u32 = 0x40000000;

#[repr(align(16))]
struct MailboxBuffer([u32; 36]);

pub fn read_mailbox(channel: u32) -> u32 {
    // Wait until mailbox is not empty
    loop {
        while (mmio_read(MBOX_READ_STATUS) & MAIL_EMPTY != 0) {}
        let mut data = mmio_read(MBOX_READ);
        let read_channel = data & 0xF;

        data >>= 4;

        if channel == read_channel {
            return data;
        }
    }
}

pub fn write_mailbox(channel: u32, data: u32) {
    while (mmio_read(MBOX_WRITE_STATUS) & MAIL_FULL != 0) {}
    mmio_write(MBOX_WRITE, data << 4 | (channel & 0xF));
}

pub fn read_soc_temp() -> u32 {
    let mut mbox = MailboxBuffer([0; 36]);

    mbox.0[0] = 8 * 4; // Total size in bytes
    mbox.0[1] = 0; // Request
    mbox.0[2] = 0x00030006; // Tag: Get temperature
    mbox.0[3] = 8; // Value buffer size (bytes)
    mbox.0[4] = 4; // Request size (bytes)
    mbox.0[5] = 0; // Temp ID: 0 = SoC
    mbox.0[6] = 0; // Response will be written here
    mbox.0[7] = 0; // End tag

    let addr = &mbox.0 as *const u32 as u32;
    write_mailbox(8, addr);
    let _ = read_mailbox(8);
    let raw_temp = mbox.0[6];
    raw_temp / 1000
}
