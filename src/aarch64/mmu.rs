use core::ptr::write_volatile;

use crate::{println, PERIPHERAL_BASE};

unsafe extern "C" {
    static mut __translation_table_l1_start: u64;
    static mut __translation_table_l2_start: u64;
    static __stack_start_el0: u64;
    static _data: u64;
}

pub fn init_translation_table() {
    unsafe {
        write_volatile(
            &raw mut __translation_table_l1_start,
            table_descriptor_entry(&raw mut __translation_table_l2_start as u64),
        );
        println!("{}", &raw mut __translation_table_l2_start as u64);

        for i in 0..512 {
            let addr = 0x0 + (i as u64 * 2 * 1024 * 1024);

            let descriptor = if addr < &_data as *const _ as u64 {
                block_descriptor_entry(addr, NORMAL_MEM, USER_AP | DISALLOW_KERNEL_AP)
            } else if addr < PERIPHERAL_BASE as u64 {
                block_descriptor_entry(addr, NORMAL_MEM, KERNEL_AP)
            } else {
                block_descriptor_entry(addr, DEVICE_MEM, USER_AP)
            };

            write_volatile(
                (&raw mut __translation_table_l2_start).byte_add(8 * i),
                descriptor,
            );
        }
    }
}

const BLOCK: u64 = 0b01;
const TABLE: u64 = 0b11;

const USER_AP: u64 = 1 << 6;
const KERNEL_AP: u64 = 0 << 7;
const DISALLOW_KERNEL_AP: u64 = 1 << 7;
const ACCESS_FLAG: u64 = 1 << 10;
const INNER_SHAREABILITY: u64 = 0b11 << 8;

const NORMAL_MEM: u64 = 0 << 2;
const DEVICE_MEM: u64 = 1 << 2;

pub fn block_descriptor_entry(addr: u64, mair_index: u64, additional_flags: u64) -> u64 {
    let pxn = 0 << 53; // allow EL1 execution
    let uxn = 0 << 54; // allow EL0 execution

    (addr & 0x0000_FFFF_FFE0_0000)
        | BLOCK
        | mair_index
        | ACCESS_FLAG
        | pxn
        | uxn
        | INNER_SHAREABILITY
        | additional_flags
}

pub fn table_descriptor_entry(addr: u64) -> u64 {
    0 | (addr & 0x0000_FFFF_FFFF_F000) | TABLE
}
