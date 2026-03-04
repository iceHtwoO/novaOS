use core::arch::asm;

pub fn init_mmu() {
    let ips = 0b000 << 32;

    // 4KB granularity
    let tg0 = 0b00 << 14;
    let tg1 = 0b00 << 30;

    //64-25 = 29 bits of VA
    // FFFF_FF80_0000_0000 start address
    let t0sz = 25;

    let tcr_el1: u64 = ips | tg0 | tg1 | t0sz;

    unsafe { asm!("msr TCR_EL1, {0:x}", in(reg) tcr_el1) };
}
