use alloc::vec::Vec;

use crate::{
    aarch64::mmu::{
        find_free_kerne_page_in_block, map_page, physical_mapping::reserve_page, TableEntry,
        NORMAL_MEM, TRANSLATIONTABLE_TTBR1, WRITABLE,
    },
    configuration::memory_mapping::APPLICATION_TRANSLATION_TABLE_VA,
};
pub struct Application {
    pub table_ptr: *mut TableEntry,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        let physical_addr = reserve_page();
        let virtual_address =
            find_free_kerne_page_in_block(APPLICATION_TRANSLATION_TABLE_VA).unwrap();

        map_page(
            virtual_address,
            physical_addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR1),
            NORMAL_MEM | WRITABLE,
        )
        .unwrap();

        Self {
            table_ptr: physical_addr as *mut TableEntry,
        }
    }
}

pub static mut APPLICATION_LIST: Option<Vec<Application>> = None;

pub fn initialize_app_manager() {
    unsafe { APPLICATION_LIST = Some(Vec::new()) }
}
