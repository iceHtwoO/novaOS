use core::ptr::{read_volatile, write_volatile};

use crate::PERIPHERAL_BASE;

/// Power Management Base
static PM_BASE: u32 = PERIPHERAL_BASE + 0x10_0000;
static PM_RSTC: u32 = PM_BASE + 0x1c;
static PM_WDOG: u32 = PM_BASE + 0x24;

static PM_PASSWORD: u32 = 0x5a000000;
static PM_WDOG_TIMER_MASK: u32 = 0x000fffff;
static PM_RSTC_WRCFG_CLR: u32 = 0xffffffcf;
static PM_RSTC_WRCFG_FULL_RESET: u32 = 0x00000020;

pub fn reboot_system() {
    unsafe {
        let pm_rstc_val = read_volatile(PM_RSTC as *mut u32);
        // (31:16) bits -> password
        // (11:0) bits -> value
        write_volatile(PM_WDOG as *mut u32, PM_PASSWORD | (1 & PM_WDOG_TIMER_MASK));
        write_volatile(
            PM_RSTC as *mut u32,
            PM_PASSWORD | (pm_rstc_val & PM_RSTC_WRCFG_CLR) | PM_RSTC_WRCFG_FULL_RESET,
        );
    }
    loop {}
}
