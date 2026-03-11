use core::u64::MAX;

use nova_error::NovaError;

use crate::{println, PERIPHERAL_BASE};

unsafe extern "C" {
    static mut __translation_table_l2_start: u64;
    static __stack_start_el0: u64;
    static __kernel_end: u64;
    static _data: u64;
}

const BLOCK: u64 = 0b01;
const TABLE: u64 = 0b11;

const EL0_ACCESSIBLE: u64 = 1 << 6;

const WRITABLE: u64 = 0 << 7;
const READ_ONLY: u64 = 1 << 7;

const ACCESS_FLAG: u64 = 1 << 10;
const INNER_SHAREABILITY: u64 = 0b11 << 8;

const NORMAL_MEM: u64 = 0 << 2;
const DEVICE_MEM: u64 = 1 << 2;

/// Disallow EL1 Execution.
const PXN: u64 = 1 << 53;

/// Disallow EL0 Execution.
const UXN: u64 = 1 << 54;

const GRANULARITY: usize = 4 * 1024;
const TABLE_ENTRY_COUNT: usize = GRANULARITY / size_of::<u64>(); // 2MiB
const LEVEL2_BLOCK_SIZE: usize = TABLE_ENTRY_COUNT * GRANULARITY;

const MAX_PAGE_COUNT: usize = 1 * 1024 * 1024 * 1024 / GRANULARITY;
#[repr(align(4096))]
pub struct PageTable([u64; TABLE_ENTRY_COUNT]);

#[no_mangle]
pub static mut TRANSLATIONTABLE_TTBR0: PageTable = PageTable([0; 512]);
pub static mut TRANSLATIONTABLE_TTBR0_L2_0: PageTable = PageTable([0; 512]);

static mut PAGING_BITMAP: [u64; MAX_PAGE_COUNT / 64] = [0; MAX_PAGE_COUNT / 64];

pub fn init_translation_table() {
    unsafe {
        TRANSLATIONTABLE_TTBR0.0[0] =
            table_descriptor_entry(&raw mut TRANSLATIONTABLE_TTBR0_L2_0 as usize);
        println!("{}", &raw mut TRANSLATIONTABLE_TTBR0_L2_0 as u64);
        println!("{}", TRANSLATIONTABLE_TTBR0.0[0] & 0x0000_FFFF_FFFF_F000);

        for i in 0..512 {
            let addr = 0x0 + (i * LEVEL2_BLOCK_SIZE);

            if addr < &_data as *const _ as usize {
                let _ = alloc_block_l2(
                    addr,
                    &TRANSLATIONTABLE_TTBR0,
                    EL0_ACCESSIBLE | READ_ONLY | NORMAL_MEM,
                );
            } else if addr < &__kernel_end as *const _ as usize {
                let _ = alloc_block_l2(addr, &TRANSLATIONTABLE_TTBR0, WRITABLE | UXN | NORMAL_MEM);
            } else if addr < PERIPHERAL_BASE {
                let _ = alloc_block_l2(
                    addr,
                    &TRANSLATIONTABLE_TTBR0,
                    EL0_ACCESSIBLE | WRITABLE | PXN | NORMAL_MEM,
                );
            } else {
                let _ = alloc_block_l2(
                    addr,
                    &TRANSLATIONTABLE_TTBR0,
                    EL0_ACCESSIBLE | WRITABLE | UXN | PXN | DEVICE_MEM,
                );
            };
        }
        println!("Done");
    }
}

pub fn alloc_page() -> Result<usize, NovaError> {
    find_unallocated_page()
}

fn find_unallocated_page() -> Result<usize, NovaError> {
    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry != u64::MAX {
            for offset in 0..64 {
                if entry >> offset & 0b1 == 0 {
                    return Ok((i * 64 + offset) * GRANULARITY);
                }
            }
        }
    }
    Err(NovaError::Paging)
}

pub fn alloc_block_l2(
    virtual_addr: usize,
    base_table: &PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    let physical_address = find_unallocated_block_l2()?;

    let l2_off = virtual_addr / GRANULARITY / TABLE_ENTRY_COUNT;
    let l1_off = l2_off / TABLE_ENTRY_COUNT;

    let l2_table =
        unsafe { &mut *((base_table.0[l1_off] & 0x0000_FFFF_FFFF_F000) as *mut PageTable) };

    let new_entry = create_block_descriptor_entry(physical_address, additional_flags);

    l2_table.0[l2_off] = new_entry;

    allocate_block_l2(physical_address);

    Ok(())
}

fn find_unallocated_block_l2() -> Result<usize, NovaError> {
    let mut count = 0;
    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry == 0 {
            count += 1;
        } else {
            count = 0;
        }

        if count == 8 {
            return Ok((i - 7) * 64 * GRANULARITY);
        }
    }
    Err(NovaError::Paging)
}

fn allocate_block_l2(physical_address: usize) {
    let page = physical_address / GRANULARITY;
    for i in 0..8 {
        unsafe { PAGING_BITMAP[(page / 64) + i] = MAX };
    }
}

fn create_block_descriptor_entry(addr: usize, additional_flags: u64) -> u64 {
    let pxn = 0 << 53; // Privileged execute never
    let uxn = 0 << 54; // Unprivileged execute never

    (addr as u64 & 0x0000_FFFF_FFE0_0000)
        | BLOCK
        | ACCESS_FLAG
        | pxn
        | uxn
        | INNER_SHAREABILITY
        | additional_flags
}

pub fn table_descriptor_entry(addr: usize) -> u64 {
    0 | (addr as u64 & 0x0000_FFFF_FFFF_F000) | TABLE
}
