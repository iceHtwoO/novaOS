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
const SH1: u64 = 0b11 << 28; // Inner sharable

const IPS: u64 = 0b000 << 32; // 32 bits of PA space -> up to 4GiB
const AS: u64 = 0b1 << 36; // configure an ASID size of 16 bits

#[no_mangle]
pub static TCR_EL1_CONF: u64 = IPS | TG0 | TG1 | T0SZ | T1SZ | SH0 | SH1 | AS;

pub mod memory_mapping;
