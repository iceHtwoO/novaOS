const TIMER_CLO: u32 = 0x3F00_3004;

fn read_clo() -> u32 {
    unsafe { return core::ptr::read_volatile(TIMER_CLO as *const u32) }
}

/// Sleep for `us` microseconds
pub fn sleep_us(us: u32) {
    let start = read_clo();
    while read_clo() - start < us {
        unsafe { core::arch::asm!("nop") }
    }
}

/// Sleep for `ms` milliseconds
pub fn sleep_ms(ms: u32) {
    sleep_us(ms * 1000);
}

/// Sleep for `s` seconds
pub fn sleep_s(s: u32) {
    sleep_us(s * 1000);
}

/// Wait for `count` operations to pass
pub fn delay_nops(count: u32) {
    for _ in 0..count {
        unsafe { core::arch::asm!("nop") }
    }
}
