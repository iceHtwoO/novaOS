use crate::{
    aarch64::mmu::{
        find_free_kerne_page_in_block, map_page, physical_mapping::reserve_page, PageTable,
        TableEntry, VirtAddr, NORMAL_MEM, TRANSLATIONTABLE_TTBR0, TRANSLATIONTABLE_TTBR1, WRITABLE,
    },
    configuration::memory_mapping::{APPLICATION_TRANSLATION_TABLE_VA, EL0_STACK_TOP},
};
use alloc::vec::Vec;
use core::{arch::asm, mem, ptr::write_volatile};
use log::error;
use nova_error::NovaError;
use spin::Mutex;

pub struct Application {
    pub table_ptr: *mut TableEntry,
    pub start_addr: usize,
}

impl Application {
    pub fn new(start_addr: VirtAddr) -> Self {
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

        // TODO: Temporary solution, while kernel and app share some memory regions
        #[allow(static_mut_refs)]
        unsafe {
            let table = &mut *(virtual_address as *mut PageTable);
            table.0 = TRANSLATIONTABLE_TTBR0.0;
        }

        Self {
            table_ptr: physical_addr as *mut TableEntry,
            start_addr,
        }
    }

    /// Starts an application.
    ///
    /// `ELR_EL1` ->  Exception Link Register (starting virtual address)
    /// `SPSR_EL1` -> Saved Program State Register (settings for `eret` behaviour)
    /// `SP_EL0` -> Stack Pointer Register (virtual_address of stack Pointer)
    /// `TTBR0_EL1` -> Translation Table base Register Register
    pub fn start(&self, args: Vec<&str>) {
        let size = args.len();

        let mut sp = EL0_STACK_TOP;
        let mut arg_addresses = Vec::with_capacity(size);
        for value in args {
            sp -= value.len() * mem::size_of::<u8>();
            let pointer = sp as *mut u8;
            unsafe { core::ptr::copy(value.as_ptr(), pointer, value.len()) };
            arg_addresses.push(pointer);
        }
        sp = align_down(sp, 16);

        let argv = sp;

        for addr in arg_addresses {
            unsafe { write_volatile(sp as *mut *const u8, addr) };
            sp -= mem::size_of::<*const u8>();
        }

        unsafe {
            asm!("msr ELR_EL1, {}", in(reg) self.start_addr);
            asm!("msr SPSR_EL1, {0:x}", in(reg) 0);
            asm!("msr SP_EL0, {0:x}", in(reg) sp);
            asm!("msr TTBR0_EL1, {}", in(reg) self.table_ptr as usize);
            asm!("", in("x0") size);
            asm!("", in("x1") argv);
            asm!("eret");
        }
    }
}

fn align_down(sp: usize, align: usize) -> usize {
    sp & !(align - 1)
}

struct AppManager {
    apps: Option<Vec<Application>>,
}

impl AppManager {
    const fn new() -> Self {
        Self { apps: None }
    }
}

unsafe impl Send for AppManager {}

static APP_MANAGER: Mutex<AppManager> = Mutex::new(AppManager::new());

pub fn initialize_app_manager() {
    let mut guard = APP_MANAGER.lock();
    guard.apps = Some(Vec::new());
}

pub fn add_app(app: Application) -> Result<(), NovaError> {
    if let Some(app_list) = APP_MANAGER.lock().apps.as_mut() {
        app_list.push(app);
        Ok(())
    } else {
        Err(NovaError::General("AppManager not initalized."))
    }
}
pub fn start_app(index: usize, args: Vec<&str>) -> Result<(), NovaError> { if let Some(app) = APP_MANAGER
        .lock()
        .apps
        .as_mut()
        .and_then(|am| am.get(index))
    {
        app.start(args);
        unreachable!()
    } else {
        error!("Unable to start app due to invalid App ID.");
        Err(NovaError::General("Invalid app id."))
    }
}
