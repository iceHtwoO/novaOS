use crate::{mmio_read, mmio_write, NovaError};

const MBOX_BASE: u32 = 0x3F00_0000 + 0xB880;

// MB0
const MBOX_READ: u32 = MBOX_BASE + 0x00;
const MBOX_STATUS: u32 = MBOX_BASE + 0x18;

// MB1
const MBOX_WRITE: u32 = MBOX_BASE + 0x20;

// Status
const MAIL_FULL: u32 = 0x80000000;
const MAIL_EMPTY: u32 = 0x40000000;

const HEADER_LENGTH: usize = 4 + 4 + 4 + 4 + 4; // Total Size + Request + Tag + MaxBufferLength + RequestLength
const FOOTER_LENGTH: usize = 4;

macro_rules! max {
    ($a:expr, $b:expr) => {{
        const M: usize = if $a > $b { $a as usize } else { $b as usize };
        M
    }};
}

#[macro_export]
macro_rules! mailbox_command {
    ($name:ident, $tag:expr, $request_len:expr,$response_len:expr) => {
        /// More information at: https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
        pub fn $name(
            request_data: [u32; $request_len / 4],
        ) -> Result<[u32; $response_len / 4], NovaError> {
            let mut mailbox =
                [0u32; (HEADER_LENGTH + max!($request_len, $response_len) + FOOTER_LENGTH) / 4];
            mailbox[0] = (HEADER_LENGTH + max!($request_len, $response_len) + FOOTER_LENGTH) as u32; // Total length in Bytes
            mailbox[1] = 0; // Request
            mailbox[2] = $tag; // Command Tag
            mailbox[3] = max!($request_len, $response_len) as u32; // Max value buffer size
            mailbox[4] = $request_len;

            mailbox[5..(5 + ($request_len / 4))].copy_from_slice(&request_data);
            mailbox[(5 + ($request_len / 4))..].fill(0);

            let addr = core::ptr::addr_of!(mailbox[0]) as u32;

            write_mailbox(8, addr);

            let _ = read_mailbox(8);

            if mailbox[1] == 0 {
                return Err(NovaError::Mailbox);
            }

            let mut out = [0u32; $response_len / 4]; // TODO: Can this be improved?
            out.copy_from_slice(&mailbox[5..(5 + $response_len / 4)]);
            Ok(out)
        }
    };
}

mailbox_command!(mb_read_soc_temp, 0x00030006, 4, 8);

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
