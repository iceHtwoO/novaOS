use core::{panic, u64::MAX};

use nova_error::NovaError;

unsafe extern "C" {
    static mut __translation_table_l2_start: u64;
    static __stack_start_el0: u64;
    static __kernel_end: u64;
    static _data: u64;
}

const BLOCK: u64 = 0b01;
const TABLE: u64 = 0b11;

pub const EL0_ACCESSIBLE: u64 = 1 << 6;

pub const WRITABLE: u64 = 0 << 7;
pub const READ_ONLY: u64 = 1 << 7;

const ACCESS_FLAG: u64 = 1 << 10;
const INNER_SHAREABILITY: u64 = 0b11 << 8;

pub const NORMAL_MEM: u64 = 0 << 2;
pub const DEVICE_MEM: u64 = 1 << 2;

/// Disallow EL1 Execution.
pub const PXN: u64 = 1 << 53;

/// Disallow EL0 Execution.
pub const UXN: u64 = 1 << 54;

pub const GRANULARITY: usize = 4 * 1024;
const TABLE_ENTRY_COUNT: usize = GRANULARITY / size_of::<u64>(); // 2MiB

pub const LEVEL1_BLOCK_SIZE: usize = TABLE_ENTRY_COUNT * TABLE_ENTRY_COUNT * GRANULARITY;
pub const LEVEL2_BLOCK_SIZE: usize = TABLE_ENTRY_COUNT * GRANULARITY;

const L2_BLOCK_BITMAP_WORDS: usize = LEVEL2_BLOCK_SIZE / (64 * GRANULARITY);

const MAX_PAGE_COUNT: usize = 1 * 1024 * 1024 * 1024 / GRANULARITY;
#[repr(align(4096))]
pub struct PageTable([u64; TABLE_ENTRY_COUNT]);

#[no_mangle]
pub static mut TRANSLATIONTABLE_TTBR0: PageTable = PageTable([0; 512]);

static mut PAGING_BITMAP: [u64; MAX_PAGE_COUNT / 64] = [0; MAX_PAGE_COUNT / 64];

pub fn allocate_memory(
    mut virtual_address: usize,
    mut size: usize,
    additional_flags: u64,
) -> Result<(), NovaError> {
    if virtual_address % GRANULARITY != 0 {
        return Err(NovaError::Misalignment);
    }

    let level1_blocks = size / LEVEL1_BLOCK_SIZE;
    size %= LEVEL1_BLOCK_SIZE;
    let level2_blocks = size / LEVEL2_BLOCK_SIZE;
    size %= LEVEL2_BLOCK_SIZE;
    let level3_pages = size / GRANULARITY;
    if size % GRANULARITY != 0 {
        return Err(NovaError::InvalidGranularity);
    }

    if level1_blocks > 0 {
        todo!("Currently not supported");
    }

    for _ in 0..level2_blocks {
        unsafe {
            alloc_block_l2(
                virtual_address,
                &mut TRANSLATIONTABLE_TTBR0,
                additional_flags,
            )?;
        }
        virtual_address += LEVEL2_BLOCK_SIZE;
    }
    for _ in 0..level3_pages {
        unsafe {
            alloc_page(
                virtual_address,
                &mut TRANSLATIONTABLE_TTBR0,
                additional_flags,
            )?;
        }
        virtual_address += GRANULARITY;
    }

    Ok(())
}
pub fn allocate_memory_explicit(
    mut virtual_address: usize,
    mut size: usize,
    mut physical_address: usize,
    additional_flags: u64,
) -> Result<(), NovaError> {
    if virtual_address % GRANULARITY != 0 {
        return Err(NovaError::Misalignment);
    }

    let level1_blocks = size / LEVEL1_BLOCK_SIZE;
    size %= LEVEL1_BLOCK_SIZE;
    let level2_blocks = size / LEVEL2_BLOCK_SIZE;
    size %= LEVEL2_BLOCK_SIZE;
    let level3_pages = size / GRANULARITY;
    if size % GRANULARITY != 0 {
        return Err(NovaError::InvalidGranularity);
    }

    if level1_blocks > 0 {
        todo!("Currently not supported");
    }

    for _ in 0..level2_blocks {
        unsafe {
            alloc_block_l2_explicit(
                virtual_address,
                physical_address,
                &mut TRANSLATIONTABLE_TTBR0,
                additional_flags,
            )?;
        }
        virtual_address += LEVEL2_BLOCK_SIZE;
        physical_address += LEVEL2_BLOCK_SIZE;
    }
    for _ in 0..level3_pages {
        unsafe {
            alloc_page_explicit(
                virtual_address,
                physical_address,
                &mut TRANSLATIONTABLE_TTBR0,
                additional_flags,
            )?;
        }
        virtual_address += GRANULARITY;
        physical_address += GRANULARITY;
    }

    Ok(())
}

pub fn alloc_page(
    virtual_address: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    map_page(
        virtual_address,
        reserve_page(),
        base_table,
        additional_flags,
    )
}

pub fn alloc_page_explicit(
    virtual_address: usize,
    physical_address: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    reserve_page_explicit(physical_address)?;
    map_page(
        virtual_address,
        physical_address,
        base_table,
        additional_flags,
    )
}

fn map_page(
    virtual_address: usize,
    physical_address: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    let (l1_off, l2_off, l3_off) = virtual_address_to_table_offset(virtual_address);

    let table = navigate_table(base_table, [l1_off, l2_off, 0], 2)?;

    if table.0[l3_off] & 0b11 > 0 {
        return Err(NovaError::Paging);
    }

    table.0[l3_off] = create_block_descriptor_entry(physical_address, additional_flags);

    Ok(())
}

pub fn alloc_block_l2(
    virtual_addr: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    map_l2_block(virtual_addr, reserve_block(), base_table, additional_flags)
}

pub fn alloc_block_l2_explicit(
    virtual_addr: usize,
    physical_address: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    reserve_block_explicit(physical_address)?;
    map_l2_block(virtual_addr, physical_address, base_table, additional_flags)
}

pub fn map_l2_block(
    virtual_addr: usize,
    physical_address: usize,
    base_table: &mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    let (l1_off, l2_off, _) = virtual_address_to_table_offset(virtual_addr);

    let table = navigate_table(base_table, [l1_off, 0, 0], 1)?;

    // Verify virtual address is available.
    if table.0[l2_off] & 0b11 != 0 {
        return Err(NovaError::Paging);
    }

    let new_entry = create_block_descriptor_entry(physical_address, additional_flags);

    table.0[l2_off] = new_entry;

    Ok(())
}
pub fn reserve_range_explicit(
    start_physical_address: usize,
    end_physical_address: usize,
) -> Result<(), NovaError> {
    let mut size = end_physical_address - start_physical_address;
    let l1_blocks = size / LEVEL1_BLOCK_SIZE;
    size %= LEVEL1_BLOCK_SIZE;
    let l2_blocks = size / LEVEL2_BLOCK_SIZE;
    size %= LEVEL2_BLOCK_SIZE;
    let l3_pages = size / GRANULARITY;

    if size % GRANULARITY != 0 {
        return Err(NovaError::Misalignment);
    }

    if l1_blocks > 0 {
        todo!();
    }

    let mut addr = start_physical_address;
    for _ in 0..l2_blocks {
        reserve_block_explicit(addr)?;
        addr += LEVEL2_BLOCK_SIZE;
    }

    for _ in 0..l3_pages {
        reserve_page_explicit(addr)?;
        addr += GRANULARITY;
    }

    Ok(())
}

fn reserve_page() -> usize {
    if let Some(address) = find_unallocated_page() {
        let page = address / GRANULARITY;
        let word_index = page / 64;
        unsafe { PAGING_BITMAP[word_index] |= 1 << (page % 64) };
        return address;
    }
    panic!("Out of Memory!");
}

fn reserve_page_explicit(physical_address: usize) -> Result<(), NovaError> {
    let page = physical_address / GRANULARITY;
    let word_index = page / 64;

    if unsafe { PAGING_BITMAP[word_index] } & (1 << (page % 64)) > 0 {
        return Err(NovaError::Paging);
    }

    unsafe { PAGING_BITMAP[word_index] |= 1 << (page % 64) };
    Ok(())
}

fn reserve_block() -> usize {
    if let Some(start) = find_contiguous_free_bitmap_words(L2_BLOCK_BITMAP_WORDS) {
        for j in 0..L2_BLOCK_BITMAP_WORDS {
            unsafe { PAGING_BITMAP[start + j] = MAX };
        }
        return start * 64 * GRANULARITY;
    }

    panic!("Out of Memory!");
}

fn reserve_block_explicit(physical_address: usize) -> Result<(), NovaError> {
    let page = physical_address / GRANULARITY;
    for i in 0..L2_BLOCK_BITMAP_WORDS {
        unsafe {
            if PAGING_BITMAP[(page / 64) + i] != 0 {
                return Err(NovaError::Paging);
            }
        };
    }
    for i in 0..L2_BLOCK_BITMAP_WORDS {
        unsafe {
            PAGING_BITMAP[(page / 64) + i] = MAX;
        };
    }
    Ok(())
}

fn create_block_descriptor_entry(physical_address: usize, additional_flags: u64) -> u64 {
    (physical_address as u64 & 0x0000_FFFF_FFE0_0000)
        | BLOCK
        | ACCESS_FLAG
        | INNER_SHAREABILITY
        | additional_flags
}

fn create_table_descriptor_entry(addr: usize) -> u64 {
    0 | (addr as u64 & 0x0000_FFFF_FFFF_F000) | TABLE
}

fn virtual_address_to_table_offset(virtual_addr: usize) -> (usize, usize, usize) {
    let absolute_page_off = virtual_addr / GRANULARITY;
    let l3_off = absolute_page_off % TABLE_ENTRY_COUNT;
    let l2_off = (absolute_page_off / TABLE_ENTRY_COUNT) % TABLE_ENTRY_COUNT;
    let l1_off = (absolute_page_off / TABLE_ENTRY_COUNT / TABLE_ENTRY_COUNT) % TABLE_ENTRY_COUNT;
    (l1_off, l2_off, l3_off)
}

fn navigate_table(
    initial_table: &mut PageTable,
    offsets: [usize; 3],
    offsets_size: usize,
) -> Result<&mut PageTable, NovaError> {
    let root_table_ptr = initial_table as *mut PageTable;
    let mut table = initial_table;
    for i in 0..offsets_size {
        let offset = offsets[i];
        match table.0[offset] & 0b11 {
            0 => {
                let new_table_addr = reserve_page();

                table.0[offset] = create_table_descriptor_entry(new_table_addr);
                table =
                    unsafe { &mut *(get_table_entry_address(table.0[offset]) as *mut PageTable) };

                map_page(
                    new_table_addr,
                    new_table_addr,
                    unsafe { &mut *root_table_ptr },
                    NORMAL_MEM | WRITABLE | PXN | UXN,
                )?;
            }
            1 => return Err(NovaError::Paging),
            3 => {
                table =
                    unsafe { &mut *(get_table_entry_address(table.0[offset]) as *mut PageTable) }
            }
            _ => panic!(),
        };
    }
    Ok(table)
}

fn find_unallocated_page() -> Option<usize> {
    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry != u64::MAX {
            for offset in 0..64 {
                if entry >> offset & 0b1 == 0 {
                    return Some((i * 64 + offset) * GRANULARITY);
                }
            }
        }
    }
    None
}

fn find_contiguous_free_bitmap_words(required_words: usize) -> Option<usize> {
    let mut run_start = 0;
    let mut run_len = 0;

    for (i, entry) in unsafe { PAGING_BITMAP }.iter().enumerate() {
        if *entry == 0 {
            if run_len == 0 {
                run_start = i;
            }
            run_len += 1;

            if run_len == required_words {
                return Some(run_start);
            }
        } else {
            run_len = 0;
        }
    }

    None
}

#[inline]
fn get_table_entry_address(entry: u64) -> u64 {
    entry & 0x0000_FFFF_FFFF_F000
}
