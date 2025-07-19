use crate::{
    mailbox::{read_mailbox, write_mailbox},
    peripherals::uart::{print, print_u32},
};

const ALLOCATE_BUFFER: u32 = 0x00040001;
const GET_PHYSICAL_DISPLAY_WH: u32 = 0x00040003;
const SET_PHYSICAL_DISPLAY_WH: u32 = 0x00048003;
const SET_VIRTUAL_DISPLAY_WH: u32 = 0x00048004;
const SET_PIXEL_DEPTH: u32 = 0x00048005;
const SET_PIXEL_ORDER: u32 = 0x00048006;
const SET_FB_OFFSET: u32 = 0x00048009;

pub fn init_fb() {
    let mut mailbox = [0; 32];
    mailbox[0] = 32 * 4;
    mailbox[1] = 0;

    mailbox[2] = SET_PHYSICAL_DISPLAY_WH;
    mailbox[3] = 8;
    mailbox[4] = 8;
    mailbox[5] = 1920;
    mailbox[6] = 1200;

    mailbox[7] = SET_VIRTUAL_DISPLAY_WH;
    mailbox[8] = 8;
    mailbox[9] = 8;
    mailbox[10] = 1920;
    mailbox[11] = 1200;

    mailbox[12] = SET_PIXEL_DEPTH;
    mailbox[13] = 4;
    mailbox[14] = 4;
    mailbox[15] = 32; // 32 bit per pixel

    mailbox[16] = SET_PIXEL_ORDER;
    mailbox[17] = 4;
    mailbox[18] = 4;
    mailbox[19] = 0x1; // RGB

    mailbox[20] = SET_FB_OFFSET;
    mailbox[21] = 8;
    mailbox[22] = 8;
    mailbox[24] = 0; // X in pixels
    mailbox[25] = 0; // Y in pixels

    mailbox[26] = ALLOCATE_BUFFER;
    mailbox[27] = 8;
    mailbox[28] = 4;
    mailbox[29] = 4096; // Alignment
    mailbox[30] = 0;

    mailbox[31] = 0; // End tag

    // TODO: validate responses

    let addr = core::ptr::addr_of!(mailbox[0]) as u32;

    write_mailbox(8, addr);

    let _ = read_mailbox(8);
    if mailbox[1] == 0 {
        print("Failed\r\n");
    }

    print_u32(mailbox[29]);

    mailbox[29] = (mailbox[29] & 0x00FF_FFFF) | 0x3F00_0000;

    let mut fb: *mut u32 = mailbox[29] as *mut u32;

    fb = unsafe { fb.add(1920 * 500 + 500) };

    for x in 0..500 {
        for y in 0..10 {
            unsafe {
                *fb = 0xFFFFBB00;
                fb = fb.add(1);
            };
        }
    }
}

pub fn print_display_resolution() {
    let mut mailbox = [0; 8];
    mailbox[0] = 8 * 4;
    mailbox[1] = 0;
    mailbox[2] = GET_PHYSICAL_DISPLAY_WH;
    mailbox[3] = 8;
    mailbox[4] = 0;
    mailbox[5] = 0;
    mailbox[6] = 0;
    mailbox[7] = 0;

    let addr = core::ptr::addr_of!(mailbox[0]) as u32;

    write_mailbox(8, addr);

    let _ = read_mailbox(8);
    if mailbox[1] == 0 {
        print("Failed\r\n");
    }

    print("Width x Height: ");
    print_u32(mailbox[5]);
    print(" x ");
    print_u32(mailbox[6]);
    print("\r\n");
}
