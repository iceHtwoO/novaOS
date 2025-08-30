use core::ptr::write_volatile;

mod bitmaps;

use bitmaps::BASIC_LEGACY;

use crate::mailbox::{read_mailbox, write_mailbox};
#[repr(align(16))]
struct Mailbox([u32; 36]);

const ALLOCATE_BUFFER: u32 = 0x0004_0001;
const GET_PHYSICAL_DISPLAY_WH: u32 = 0x0004_0003;
const SET_PHYSICAL_DISPLAY_WH: u32 = 0x0004_8003;
const SET_VIRTUAL_DISPLAY_WH: u32 = 0x0004_8004;
const SET_PIXEL_DEPTH: u32 = 0x0004_8005;
const SET_PIXEL_ORDER: u32 = 0x0004_8006;
const GET_PITCH: u32 = 0x000_40008;
const SET_FB_OFFSET: u32 = 0x0004_8009;

pub struct FrameBuffer {
    pixel_depth: u32, // Bits per pixel
    pitch: u32,       // Pixel per row
    rows: u32,        // Rows
    start_addr: *mut u32,
    size: u32, //Bytes
}

pub const RED: u32 = 0x00FF0000;
pub const GREEN: u32 = 0x0000FF00;
pub const BLUE: u32 = 0x000000FF;
pub const ORANGE: u32 = 0x00FFA500;
pub const YELLOW: u32 = 0x00FFFF00;

impl FrameBuffer {
    pub fn new() -> Self {
        let mut mailbox = Mailbox([0; 36]);
        mailbox.0[0] = 35 * 4;
        mailbox.0[1] = 0;

        mailbox.0[2] = SET_PHYSICAL_DISPLAY_WH;
        mailbox.0[3] = 8;
        mailbox.0[4] = 8;
        mailbox.0[5] = 1920;
        mailbox.0[6] = 1080;

        mailbox.0[7] = SET_VIRTUAL_DISPLAY_WH;
        mailbox.0[8] = 8;
        mailbox.0[9] = 8;
        mailbox.0[10] = 1920;
        mailbox.0[11] = 1080;

        mailbox.0[12] = SET_PIXEL_DEPTH;
        mailbox.0[13] = 4;
        mailbox.0[14] = 4;
        mailbox.0[15] = 32; // 32 bit per pixel

        mailbox.0[16] = SET_PIXEL_ORDER;
        mailbox.0[17] = 4;
        mailbox.0[18] = 4;
        mailbox.0[19] = 0x0; // RGB

        mailbox.0[20] = SET_FB_OFFSET;
        mailbox.0[21] = 8;
        mailbox.0[22] = 8;
        mailbox.0[23] = 0; // X in pixels
        mailbox.0[24] = 0; // Y in pixels

        mailbox.0[25] = ALLOCATE_BUFFER;
        mailbox.0[26] = 8;
        mailbox.0[27] = 4;
        mailbox.0[28] = 4096; // Alignment
        mailbox.0[29] = 0;

        mailbox.0[30] = GET_PITCH;
        mailbox.0[31] = 4;
        mailbox.0[32] = 0;
        mailbox.0[33] = 0;

        mailbox.0[34] = 0; // End tag

        // TODO: validate responses

        let addr = core::ptr::addr_of!(mailbox.0[0]) as u32;

        write_mailbox(8, addr);

        let _ = read_mailbox(8);
        if mailbox.0[1] == 0 {
            println!("Failed");
        }

        mailbox.0[28] &= 0x3FFFFFFF;

        Self {
            pixel_depth: mailbox.0[15],
            pitch: mailbox.0[33] / (mailbox.0[15] / 8),
            rows: mailbox.0[29] / mailbox.0[33],
            start_addr: mailbox.0[28] as *mut u32,
            size: mailbox.0[29],
        }
    }

    pub fn draw_pixel(&self, x: u32, y: u32, color: u32) {
        let offset = x + y * self.pitch;
        unsafe {
            write_volatile(self.start_addr.add(offset as usize), color);
        }
    }

    /*Bresenham's line algorithm
    TODO: check if its possible to optimize y1==y2 case (ARM neon?)
    */
    pub fn draw_line(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        if x1 == x2 {
            for y in y1..=y2 {
                self.draw_pixel(x1, y, color);
            }
            return;
        }

        if (y2 as i32 - y1 as i32).abs() < (x2 as i32 - x1 as i32).abs() {
            if x1 > x2 {
                self.plot_line_low(x2, y2, x1, y1, color);
            } else {
                self.plot_line_low(x1, y1, x2, y2, color);
            }
        } else {
            if y1 > y2 {
                self.plot_line_high(x2, y2, x1, y1, color);
            } else {
                self.plot_line_high(x1, y1, x2, y2, color);
            }
        }
    }

    pub fn draw_square(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        self.draw_line(x1, y1, x2, y1, color);
        self.draw_line(x1, y2, x2, y2, color);
        self.draw_line(x1, y1, x1, y2, color);
        self.draw_line(x2, y1, x2, y2, color);
    }

    pub fn draw_square_fill(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        let mut y_start = y1;
        let mut y_end = y2;

        if y2 < y1 {
            y_start = y2;
            y_end = y1;
        }

        for y in y_start..=y_end {
            self.draw_line(x1, y, x2, y, color);
        }
    }

    fn plot_line_low(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        let dx = x2 as i32 - x1 as i32;
        let mut dy = y2 as i32 - y1 as i32;
        let mut yi = 1;

        let mut d = 2 * dy - dx;
        let mut y = y1 as i32;

        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        for x in x1..=x2 {
            self.draw_pixel(x, y as u32, color);
            if d > 0 {
                y += yi;
                d += 2 * (dy - dx);
            } else {
                d += 2 * dy;
            }
        }
    }
    fn plot_line_high(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
        let mut dx = x2 as i32 - x1 as i32;
        let dy = y2 as i32 - y1 as i32;
        let mut xi: i32 = 1;

        let mut d = 2 * dy - dx;
        let mut x = x1 as i32;

        if dx < 0 {
            xi = -1;
            dx = -dx;
        }

        for y in y1..=y2 {
            self.draw_pixel(x as u32, y, color);
            if d > 0 {
                x += xi;
                d += 2 * (dx - dy);
            } else {
                d += 2 * dx;
            }
        }
    }

    //TODO: Scale in pixels
    pub fn draw_string(&self, string: &str, x: u32, mut y: u32, scale: u32, color: u32) {
        let mut offset = 0;
        for c in string.bytes() {
            match c {
                b'\n' => {
                    y += 8 * scale;
                    offset = 0;
                }
                _ => {
                    self.draw_ascii(x + (offset as u32 * 8 * scale), y, c as usize, scale, color);
                    offset += 1
                }
            }
        }
    }

    fn draw_ascii(&self, x: u32, y: u32, char: usize, scale: u32, color: u32) {
        for (y_offset, row) in (&BASIC_LEGACY[char]).iter().enumerate() {
            for bit in 0..8 {
                match row & (1 << bit) {
                    0 => {}
                    _ => self.draw_square_fill(
                        x + (bit * scale),
                        y + (y_offset as u32 * scale),
                        x + ((bit + 1) * scale),
                        y + ((y_offset + 1) as u32 * scale),
                        color,
                    ),
                }
            }
        }
    }

    pub fn draw_function(&self, f: fn(u32) -> f64, x_offset: i32, y_offset: i32, color: u32) {
        for x in 0..self.pitch as i32 {
            let y = f(x as u32);
            self.draw_pixel((x + x_offset) as u32, (y + y_offset as f64) as u32, color);
        }
    }
}

pub fn print_display_resolution() {
    let mut mailbox: [u32; 8] = [0; 8];
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
        println!("Failed");
    }

    println!("Width x Height: {}x{}", mailbox[5], mailbox[6]);
}
