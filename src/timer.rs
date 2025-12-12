use core::{hint::spin_loop, ptr::read_volatile};

const TIMER_CLOCK_LO: u32 = 0x3F00_3004;
const TIMER_CLOCK_HI: u32 = 0x3F00_3008;

fn read_timer_32() -> u32 {
    unsafe { read_volatile(TIMER_CLOCK_LO as *const u32) }
}

fn read_timer_64() -> u64 {
    loop {
        let clock_hi1 = unsafe { read_volatile(TIMER_CLOCK_HI as *const u32) };
        let clock_lo = unsafe { read_volatile(TIMER_CLOCK_LO as *const u32) };
        let clock_hi2 = unsafe { read_volatile(TIMER_CLOCK_HI as *const u32) };

        // account for roll over during read
        if clock_hi1 == clock_hi2 {
            return ((clock_hi1 as u64) << 32) | clock_lo as u64;
        }
    }
}

/// Sleep for `us` microseconds
pub fn sleep_us(us: u64) {
    if us < u32::MAX as u64 {
        sleep_us_u32(us as u32);
    } else {
        sleep_us_u64(us);
    }
}

fn sleep_us_u32(us: u32) {
    let start = read_timer_32();
    while read_timer_32().wrapping_sub(start) < us {
        spin_loop();
    }
}

fn sleep_us_u64(us: u64) {
    let start = read_timer_64();
    while read_timer_64().wrapping_sub(start) < us {
        spin_loop();
    }
}

/// Sleep for `ms` milliseconds
pub fn sleep_ms(ms: u64) {
    sleep_us(ms * 1_000);
}

/// Sleep for `s` seconds
pub fn sleep_s(s: u64) {
    sleep_ms(s * 1_000);
}

/// Wait for `count` operations to pass
pub fn delay_nops(count: u32) {
    for _ in 0..count {
        spin_loop()
    }
}
