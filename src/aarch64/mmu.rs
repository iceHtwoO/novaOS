use core::mem::size_of;
use nova_error::NovaError;

use crate::{
    aarch64::mmu::physical_mapping::{
        reserve_block, reserve_block_explicit, reserve_page, reserve_page_explicit,
    },
    get_current_el,
};

const BLOCK: u64 = 0b01;
const TABLE: u64 = 0b11;
const PAGE: u64 = 0b11;

/// Allow EL0 to access this section
pub const EL0_ACCESSIBLE: u64 = 1 << 6;

/// Allow a page or block to be written.
pub const WRITABLE: u64 = 0 << 7;
/// Disallow a page or block to be written.
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

const MAX_PAGE_COUNT: usize = 1024 * 1024 * 1024 / GRANULARITY;

const TRANSLATION_TABLE_BASE_ADDR: usize = 0xFFFF_FF82_0000_0000;
#[no_mangle]
pub static KERNEL_VIRTUAL_MEM_SPACE: usize = 0xFFFF_FF80_0000_0000;

pub const STACK_START_ADDR: usize = !KERNEL_VIRTUAL_MEM_SPACE & (!0xF);

pub mod physical_mapping;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

#[derive(Clone, Copy)]
pub struct TableEntry {
    value: u64,
}

impl TableEntry {
    pub fn invalid() -> Self {
        Self { value: 0 }
    }

    fn table_descriptor(addr: PhysAddr) -> Self {
        Self {
            value: (addr as u64 & 0x0000_FFFF_FFFF_F000) | TABLE,
        }
    }

    fn block_descriptor(physical_address: usize, additional_flags: u64) -> Self {
        Self {
            value: (physical_address as u64 & 0x0000_FFFF_FFFF_F000)
                | BLOCK
                | ACCESS_FLAG
                | INNER_SHAREABILITY
                | additional_flags,
        }
    }

    fn page_descriptor(physical_address: usize, additional_flags: u64) -> Self {
        Self {
            value: (physical_address as u64 & 0x0000_FFFF_FFFF_F000)
                | PAGE
                | ACCESS_FLAG
                | INNER_SHAREABILITY
                | additional_flags,
        }
    }

    fn is_invalid(self) -> bool {
        self.value & 0b11 == 0
    }

    #[inline]
    fn address(self) -> PhysAddr {
        self.value as usize & 0x0000_FFFF_FFFF_F000
    }
}

pub enum PhysSource {
    Any,
    Explicit(PhysAddr),
}

#[repr(align(4096))]
pub struct PageTable(pub [TableEntry; TABLE_ENTRY_COUNT]);

impl Iterator for PageTable {
    type Item = VirtAddr;

    fn next(&mut self) -> Option<Self::Item> {
        for (offset, entity) in self.0.iter().enumerate() {
            if entity.is_invalid() {
                return Some(offset);
            }
        }
        None
    }
}
#[no_mangle]
pub static mut TRANSLATIONTABLE_TTBR0: PageTable = PageTable([TableEntry { value: 0 }; 512]);
#[no_mangle]
pub static mut TRANSLATIONTABLE_TTBR1: PageTable = PageTable([TableEntry { value: 0 }; 512]);

/// Allocate a memory block of `size` starting at `virtual_address`.
pub fn allocate_memory(
    virtual_address: usize,
    size_bytes: usize,
    phys: PhysSource,
    flags: u64,
) -> Result<(), NovaError> {
    if !virtual_address.is_multiple_of(GRANULARITY) {
        return Err(NovaError::Misalignment);
    }
    if !size_bytes.is_multiple_of(GRANULARITY) {
        return Err(NovaError::InvalidGranularity);
    }

    let base_table = if virtual_address & KERNEL_VIRTUAL_MEM_SPACE > 0 {
        core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR1)
    } else {
        core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR0)
    };

    match phys {
        PhysSource::Any => map_range_dynamic(virtual_address, size_bytes, base_table, flags),
        PhysSource::Explicit(phys_addr) => {
            map_range_explicit(virtual_address, phys_addr, size_bytes, base_table, flags)
        }
    }
}

fn map_range_explicit(
    mut virt: VirtAddr,
    mut phys: PhysAddr,
    size_bytes: usize,
    base: *mut PageTable,
    flags: u64,
) -> Result<(), NovaError> {
    let mut remaining = size_bytes;

    while !virt.is_multiple_of(LEVEL2_BLOCK_SIZE) && remaining > 0 {
        map_page(virt, phys, base, flags)?;
        (virt, _) = virt.overflowing_add(GRANULARITY);
        phys += GRANULARITY;
        remaining -= GRANULARITY;
    }

    while remaining >= LEVEL2_BLOCK_SIZE {
        map_l2_block(virt, phys, base, flags)?;
        (virt, _) = virt.overflowing_add(LEVEL2_BLOCK_SIZE);
        phys += LEVEL2_BLOCK_SIZE;
        remaining -= LEVEL2_BLOCK_SIZE;
    }

    while remaining > 0 {
        map_page(virt, phys, base, flags)?;
        (virt, _) = virt.overflowing_add(GRANULARITY);
        phys += GRANULARITY;
        remaining -= GRANULARITY;
    }

    Ok(())
}

fn map_range_dynamic(
    mut virt: PhysAddr,
    size_bytes: usize,
    base: *mut PageTable,
    flags: u64,
) -> Result<(), NovaError> {
    let mut remaining = size_bytes;

    while remaining >= LEVEL2_BLOCK_SIZE {
        map_l2_block(virt, reserve_block(), base, flags)?;
        (virt, _) = virt.overflowing_add(LEVEL2_BLOCK_SIZE);
        remaining -= LEVEL2_BLOCK_SIZE;
    }

    while remaining > 0 {
        map_page(virt, reserve_page(), base, flags)?;
        (virt, _) = virt.overflowing_add(GRANULARITY);
        remaining -= GRANULARITY;
    }
    Ok(())
}

/// Allocate a singe page.
pub fn alloc_page(
    virtual_address: VirtAddr,
    base_table: *mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    map_page(
        virtual_address,
        reserve_page(),
        base_table,
        additional_flags,
    )
}

/// Allocate a singe page in one block.
pub fn find_free_kerne_page_in_block(start: VirtAddr) -> Result<VirtAddr, NovaError> {
    if !start.is_multiple_of(LEVEL2_BLOCK_SIZE) {
        return Err(NovaError::Misalignment);
    }

    let (off1, off2, _) = virtual_address_to_table_offset(start);
    let offsets = [off1, off2];
    let table = unsafe {
        &mut *navigate_table(
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR1),
            &offsets,
            true,
        )?
    };

    if let Some(offset) = table.next() {
        return Ok(start + (offset * GRANULARITY));
    }
    Err(NovaError::OutOfMeomory)
}

/// Allocate a single page at an explicit `physical_address`.
pub fn alloc_page_explicit(
    virtual_address: usize,
    physical_address: usize,
    base_table: *mut PageTable,
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

pub fn map_page(
    virtual_address: usize,
    physical_address: usize,
    base_table_ptr: *mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    let (l1_off, l2_off, l3_off) = virtual_address_to_table_offset(virtual_address);

    let offsets = [l1_off, l2_off];

    let table_ptr = navigate_table(base_table_ptr, &offsets, true)?;
    let table = unsafe { &mut *table_ptr };

    if !table.0[l3_off].is_invalid() {
        return Err(NovaError::Paging("Page already occupied."));
    }

    table.0[l3_off] = TableEntry::page_descriptor(physical_address, additional_flags);

    Ok(())
}

// Allocate a level 2 block, at a explicit `physical_address`.
pub fn alloc_block_l2_explicit(
    virtual_addr: usize,
    physical_address: usize,
    base_table_ptr: *mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    if !physical_address.is_multiple_of(LEVEL2_BLOCK_SIZE) {
        return Err(NovaError::Misalignment);
    }

    reserve_block_explicit(physical_address)?;
    map_l2_block(
        virtual_addr,
        physical_address,
        base_table_ptr,
        additional_flags,
    )
}

pub fn map_l2_block(
    virtual_addr: usize,
    physical_address: usize,
    base_table_ptr: *mut PageTable,
    additional_flags: u64,
) -> Result<(), NovaError> {
    let (l1_off, l2_off, _) = virtual_address_to_table_offset(virtual_addr);
    let offsets = [l1_off];
    let table_ptr = navigate_table(base_table_ptr, &offsets, true)?;

    let table = unsafe { &mut *table_ptr };

    // Verify virtual address is available.
    if !table.0[l2_off].is_invalid() {
        return Err(NovaError::Paging("Block already occupied."));
    }

    let new_entry = TableEntry::block_descriptor(physical_address, additional_flags);

    table.0[l2_off] = new_entry;

    Ok(())
}

pub fn reserve_range(
    start_physical_address: PhysAddr,
    end_physical_address: PhysAddr,
) -> Result<PhysAddr, NovaError> {
    let mut size = end_physical_address - start_physical_address;
    let l1_blocks = size / LEVEL1_BLOCK_SIZE;
    size %= LEVEL1_BLOCK_SIZE;
    let l2_blocks = size / LEVEL2_BLOCK_SIZE;
    size %= LEVEL2_BLOCK_SIZE;
    let l3_pages = size / GRANULARITY;

    if !size.is_multiple_of(GRANULARITY) {
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

    Ok(start_physical_address)
}

fn virtual_address_to_table_offset(virtual_addr: usize) -> (usize, usize, usize) {
    let absolute_page_off = (virtual_addr & !KERNEL_VIRTUAL_MEM_SPACE) / GRANULARITY;
    let l3_off = absolute_page_off % TABLE_ENTRY_COUNT;
    let l2_off = (absolute_page_off / TABLE_ENTRY_COUNT) % TABLE_ENTRY_COUNT;
    let l1_off = (absolute_page_off / TABLE_ENTRY_COUNT / TABLE_ENTRY_COUNT) % TABLE_ENTRY_COUNT;
    (l1_off, l2_off, l3_off)
}

/// Navigate the table tree, by following given offsets. This function
/// allocates new tables if required.
fn navigate_table(
    initial_table_ptr: *mut PageTable,
    offsets: &[usize],
    create_missing: bool,
) -> Result<*mut PageTable, NovaError> {
    let mut table = initial_table_ptr;
    for offset in offsets {
        table = next_table(table, *offset, create_missing)?;
    }
    Ok(table)
}

/// Get the next table one level down.
///
/// If table doesn't exit a page will be allocated for it.
fn next_table(
    table_ptr: *mut PageTable,
    offset: usize,
    create_missing: bool,
) -> Result<*mut PageTable, NovaError> {
    let table = unsafe { &mut *table_ptr };
    match table.0[offset].value & 0b11 {
        0 => {
            if !create_missing {
                return Err(NovaError::Paging("No table defined."));
            }
            let new_phys_page_table_address = reserve_page();

            table.0[offset] = TableEntry::table_descriptor(new_phys_page_table_address);
            map_page(
                phys_table_to_kernel_space(new_phys_page_table_address),
                new_phys_page_table_address,
                &raw mut TRANSLATIONTABLE_TTBR1,
                NORMAL_MEM | WRITABLE | PXN | UXN,
            )?;

            Ok(resolve_table_addr(table.0[offset].address()) as *mut PageTable)
        }
        1 => Err(NovaError::Paging(
            "Can't navigate table due to block mapping.",
        )),
        3 => Ok(resolve_table_addr(table.0[offset].address()) as *mut PageTable),
        _ => unreachable!(),
    }
}

/// Converts a physical table address and returns the corresponding virtual address depending on EL.
///
/// - `== EL0` -> panic
/// - `== EL1` -> 0xFFFFFF82XXXXXXXX
/// - `>= EL2` -> physical address
#[inline]
fn resolve_table_addr(physical_address: PhysAddr) -> VirtAddr {
    let current_el = get_current_el();

    if current_el >= 2 {
        physical_address
    } else if get_current_el() == 1 {
        phys_table_to_kernel_space(physical_address)
    } else {
        panic!("Access to table entries is forbidden in EL0.")
    }
}

/// Extracts the physical address out of an table entry.
#[inline]
fn phys_table_to_kernel_space(entry: usize) -> VirtAddr {
    entry | TRANSLATION_TABLE_BASE_ADDR
}
