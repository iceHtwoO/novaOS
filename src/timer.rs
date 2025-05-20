const TIMER_CLO: u32 = 0x3F00_3004;

fn read_clo() -> u32 {
    unsafe { return core::ptr::read_volatile(TIMER_CLO as *const u32) }
}

pub unsafe fn sleep(microseconds: u32) {
    let start = read_clo();
    while read_clo() - start < microseconds {
        core::arch::asm!("nop");
    }
}

pub unsafe fn delay_nops(count: u32) {
    for _ in 0..count {
        core::arch::asm!("nop");
    }
}
