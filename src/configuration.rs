const SCTLR_EL1_MMU_ENABLED: u64 = 1; //M
const SCTLR_EL1_DATA_CACHE_DISABLED: u64 = 0 << 2; //C
const SCTLR_EL1_INSTRUCTION_CACHE_DISABLED: u64 = 0 << 12; //I
const SCTLR_EL1_LITTLE_ENDIAN_EL0: u64 = 0 << 24; //E0E
const SCTLR_EL1_LITTLE_ENDIAN_EL1: u64 = 0 << 25; //EE
const SCTLR_EL1_SPAN: u64 = 1 << 23; //SPAN

#[allow(clippy::identity_op)]
const SCTLR_EL1_RES: u64 = (0 << 6) | (1 << 11) | (0 << 17) | (1 << 20) | (1 << 22); //Res0 & Res1

#[no_mangle]
pub static SCTLR_EL1_CONF: u64 = SCTLR_EL1_MMU_ENABLED
    | SCTLR_EL1_DATA_CACHE_DISABLED
    | SCTLR_EL1_INSTRUCTION_CACHE_DISABLED
    | SCTLR_EL1_LITTLE_ENDIAN_EL0
    | SCTLR_EL1_LITTLE_ENDIAN_EL1
    | SCTLR_EL1_RES
    | SCTLR_EL1_SPAN;

const TG0: u64 = 0b00 << 14; // 4KB granularity EL0
const T0SZ: u64 = 25; // 25 Bits of TTBR select -> 39 Bits of VA
const SH0: u64 = 0b11 << 12; // Inner shareable

const TG1: u64 = 0b10 << 30; // 4KB granularity EL1
const T1SZ: u64 = 25 << 16; // 25 Bits of TTBR select -> 39 Bits of VA
const EPD1: u64 = 0b1 << 23; // Trigger translation fault when using TTBR1_EL1
const SH1: u64 = 0b11 << 28; // Inner sharable

const IPS: u64 = 0b000 << 32; // 32 bits of PA space -> up to 4GiB
const AS: u64 = 0b1 << 36; // configure an ASID size of 16 bits

#[no_mangle]
pub static TCR_EL1_CONF: u64 = IPS | TG0 | TG1 | T0SZ | T1SZ | SH0 | SH1 | EPD1 | AS;

pub mod mmu {

    use crate::{
        aarch64::mmu::{
            alloc_block_l2_explicit, map_l2_block, reserve_range_explicit, DEVICE_MEM,
            EL0_ACCESSIBLE, LEVEL1_BLOCK_SIZE, LEVEL2_BLOCK_SIZE, NORMAL_MEM, PXN, READ_ONLY,
            TRANSLATIONTABLE_TTBR0, UXN, WRITABLE,
        },
        PERIPHERAL_BASE,
    };
    extern "C" {
        static _data: u64;
        static _end: u64;
        static __kernel_end: u64;
    }

    pub fn initialize_mmu_translation_tables() {
        let shared_segment_end = unsafe { &_data } as *const _ as usize;
        let kernel_end = unsafe { &__kernel_end } as *const _ as usize;
        let user_space_end = unsafe { &_end } as *const _ as usize;

        reserve_range_explicit(0x0, user_space_end).unwrap();

        for addr in (0..shared_segment_end).step_by(LEVEL2_BLOCK_SIZE) {
            let _ = map_l2_block(
                addr,
                addr,
                unsafe { &mut TRANSLATIONTABLE_TTBR0 },
                EL0_ACCESSIBLE | READ_ONLY | NORMAL_MEM,
            );
        }

        for addr in (shared_segment_end..kernel_end).step_by(LEVEL2_BLOCK_SIZE) {
            let _ = map_l2_block(
                addr,
                addr,
                unsafe { &mut TRANSLATIONTABLE_TTBR0 },
                WRITABLE | UXN | NORMAL_MEM,
            );
        }

        for addr in (kernel_end..user_space_end).step_by(LEVEL2_BLOCK_SIZE) {
            let _ = map_l2_block(
                addr,
                addr,
                unsafe { &mut TRANSLATIONTABLE_TTBR0 },
                EL0_ACCESSIBLE | WRITABLE | PXN | NORMAL_MEM,
            );
        }

        for addr in (PERIPHERAL_BASE..LEVEL1_BLOCK_SIZE).step_by(LEVEL2_BLOCK_SIZE) {
            let _ = alloc_block_l2_explicit(
                addr,
                addr,
                unsafe { &mut TRANSLATIONTABLE_TTBR0 },
                EL0_ACCESSIBLE | WRITABLE | UXN | PXN | DEVICE_MEM,
            );
        }
    }
}
