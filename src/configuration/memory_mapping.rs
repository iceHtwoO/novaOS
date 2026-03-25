use crate::{
    aarch64::mmu::{
        alloc_block_l2_explicit, allocate_memory, map_page, physical_mapping::reserve_page,
        reserve_range, PhysAddr, PhysSource, VirtAddr, DEVICE_MEM, EL0_ACCESSIBLE, GRANULARITY,
        KERNEL_VIRTUAL_MEM_SPACE, LEVEL1_BLOCK_SIZE, LEVEL2_BLOCK_SIZE, NORMAL_MEM, PXN, READ_ONLY,
        STACK_START_ADDR, TRANSLATIONTABLE_TTBR0, TRANSLATIONTABLE_TTBR1, UXN, WRITABLE,
    },
    PERIPHERAL_BASE,
};

#[no_mangle]
static EL1_STACK_TOP: usize = STACK_START_ADDR | KERNEL_VIRTUAL_MEM_SPACE;
const EL1_STACK_SIZE: usize = LEVEL2_BLOCK_SIZE * 2;
#[no_mangle]
pub static EL0_STACK_TOP: usize = STACK_START_ADDR;
pub const EL0_STACK_SIZE: usize = LEVEL2_BLOCK_SIZE * 2;

pub const MAILBOX_VIRTUAL_ADDRESS: VirtAddr = 0xFFFF_FF81_FFFF_E000;
pub static mut MAILBOX_PHYSICAL_ADDRESS: Option<PhysAddr> = None;

// TODO: Currently limited to 512 applications, more than enough, but has to be kept
// in mind
pub const APPLICATION_TRANSLATION_TABLE_VA: VirtAddr = 0xFFFF_FF81_FE00_0000;

extern "C" {
    static __text_end: u64;
    static __share_end: u64;
    static __kernel_end: u64;
}

pub fn initialize_mmu_translation_tables() {
    let text_end = unsafe { &__text_end } as *const _ as usize;
    let shared_segment_end = unsafe { &__share_end } as *const _ as usize;
    let kernel_end = unsafe { &__kernel_end } as *const _ as usize;

    reserve_range(0x0, kernel_end).unwrap();

    for addr in (0..text_end).step_by(GRANULARITY) {
        map_page(
            addr,
            addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR0),
            EL0_ACCESSIBLE | READ_ONLY | NORMAL_MEM,
        )
        .unwrap();
    }

    for addr in (0..text_end).step_by(GRANULARITY) {
        map_page(
            addr | KERNEL_VIRTUAL_MEM_SPACE,
            addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR1),
            READ_ONLY | NORMAL_MEM,
        )
        .unwrap();
    }

    for addr in (text_end..shared_segment_end).step_by(GRANULARITY) {
        map_page(
            addr,
            addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR0),
            EL0_ACCESSIBLE | WRITABLE | NORMAL_MEM,
        )
        .unwrap();
    }

    for addr in (text_end..shared_segment_end).step_by(GRANULARITY) {
        map_page(
            addr | KERNEL_VIRTUAL_MEM_SPACE,
            addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR1),
            EL0_ACCESSIBLE | WRITABLE | NORMAL_MEM,
        )
        .unwrap();
    }

    for addr in (PERIPHERAL_BASE..LEVEL1_BLOCK_SIZE).step_by(LEVEL2_BLOCK_SIZE) {
        alloc_block_l2_explicit(
            addr,
            addr,
            core::ptr::addr_of_mut!(TRANSLATIONTABLE_TTBR0),
            EL0_ACCESSIBLE | WRITABLE | UXN | PXN | DEVICE_MEM,
        )
        .unwrap();
    }

    // Frame Buffer memory range
    allocate_memory(
        0x3c100000,
        1080 * 1920 * 4,
        PhysSource::Explicit(0x3c100000),
        NORMAL_MEM | PXN | UXN | WRITABLE | EL0_ACCESSIBLE,
    )
    .unwrap();

    // Allocate EL1 stack
    allocate_memory(
        EL1_STACK_TOP - EL1_STACK_SIZE + 0x10,
        EL1_STACK_SIZE,
        PhysSource::Any,
        WRITABLE | NORMAL_MEM,
    )
    .unwrap();

    // Allocate EL0 stack
    allocate_memory(
        EL0_STACK_TOP - EL0_STACK_SIZE + 0x10,
        EL0_STACK_SIZE,
        PhysSource::Any,
        WRITABLE | EL0_ACCESSIBLE | NORMAL_MEM,
    )
    .unwrap();

    // Allocate Mailbox buffer
    {
        let addr = reserve_page();
        unsafe { MAILBOX_PHYSICAL_ADDRESS = Some(addr) };
        allocate_memory(
            MAILBOX_VIRTUAL_ADDRESS,
            GRANULARITY,
            PhysSource::Explicit(addr),
            WRITABLE | NORMAL_MEM,
        )
        .unwrap();
    }
}
