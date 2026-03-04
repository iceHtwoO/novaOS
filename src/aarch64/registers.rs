use core::arch::asm;

pub mod daif {
    use core::arch::asm;

    #[inline(always)]
    pub fn mask_all() {
        unsafe { asm!("msr DAIFSet, #0xf", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn unmask_all() {
        unsafe { asm!("msr DAIFClr, #0xf", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn mask_irq() {
        unsafe { asm!("msr DAIFSet, #0x2", options(nomem, nostack)) }
    }

    #[inline(always)]
    pub fn unmask_irq() {
        unsafe { asm!("msr DAIFClr, #0x2", options(nomem, nostack)) }
    }
}

#[macro_export]
macro_rules! psr {
    ($name:ident, $t:tt) => {
        paste::item! {
            pub fn [<read_ $name:lower>]() -> $t {
                let buf: $t;
                unsafe {
                    asm!(
                        concat!("mrs {0:x}, ", stringify!($name)),
                        out(reg) buf
                    );
                }
                buf
            }
        }
    };
}

psr!(TCR_EL1, u64);

psr!(ID_AA64MMFR0_EL1, u64);

psr!(ESR_EL1, u32);

psr!(SPSR_EL1, u32);

psr!(ELR_EL1, u32);

pub fn read_exception_source_el() -> u32 {
    read_spsr_el1() & 0b1111
}
