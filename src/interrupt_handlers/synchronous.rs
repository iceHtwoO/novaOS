use crate::{
    aarch64::registers::{daif::mask_all, read_elr_el1, read_esr_el1, read_exception_source_el},
    get_current_el,
    interrupt_handlers::{set_return_to_kernel_loop, EsrElX, TrapFrame},
    pi3::mailbox,
};

use log::{debug, error, warn};

/// Synchronous Exception Handler
///
/// Source is a lower Exception level, where the implemented level
/// immediately lower than the target level is using
/// AArch64.
#[no_mangle]
unsafe extern "C" fn rust_synchronous_interrupt_imm_lower_aarch64(frame: &mut TrapFrame) -> usize {
    mask_all();
    let esr: EsrElX = EsrElX::from(read_esr_el1());
    debug!("Synchronous interrupt from lower EL triggered");
    log_sync_exception();
    match esr.ec {
        0b100100 => {
            error!("Data Abort from a lower Exception level");
            error!("Cause: {}", decode_data_abort(esr.iss as usize));
        }
        0b010101 => {
            debug!("SVC instruction execution in AArch64");
            return handle_svc(frame);
        }
        0b100010 => {
            error!("PC alignment fault.");
        }
        _ => {
            error!("Synchronous interrupt: Unknown Error Code: {:b}", esr.ec);
        }
    }

    warn!("UnhandledException -> Returning to kernel...");
    set_return_to_kernel_loop();
    0
}

fn decode_data_abort(iss: usize) -> &'static str {
    match iss & 0b111111 {
        0b000000 => "Address size fault, level 0",
        0b000001 => "Address size fault, level 1",
        0b000010 => "Address size fault, level 2",
        0b000011 => "Address size fault, level 3",

        0b000100 => "Translation fault, level 0",
        0b000101 => "Translation fault, level 1",
        0b000110 => "Translation fault, level 2",
        0b000111 => "Translation fault, level 3",

        0b001001 => "Access flag fault, level 1",
        0b001010 => "Access flag fault, level 2",
        0b001011 => "Access flag fault, level 3",

        0b001101 => "Permission fault, level 1",
        0b001110 => "Permission fault, level 2",
        0b001111 => "Permission fault, level 3",

        0b010000 => "Synchronous External abort, not on translation table walk",
        0b011000 => {
            "Synchronous parity or ECC error on memory access, not on translation table walk"
        }

        0b010100 => "Synchronous External abort, on translation table walk, level 0",
        0b010101 => "Synchronous External abort, on translation table walk, level 1",
        0b010110 => "Synchronous External abort, on translation table walk, level 2",
        0b010111 => "Synchronous External abort, on translation table walk, level 3",

        0b011100 => "Synchronous parity or ECC error on translation table walk, level 0",
        0b011101 => "Synchronous parity or ECC error on translation table walk, level 1",
        0b011110 => "Synchronous parity or ECC error on translation table walk, level 2",
        0b011111 => "Synchronous parity or ECC error on translation table walk, level 3",

        0b100001 => "Alignment fault",
        0b110000 => "TLB conflict abort",
        0b110001 => "Unsupported atomic hardware update fault",

        0b110100 => "IMPLEMENTATION DEFINED fault (Lockdown)",
        0b110101 => "IMPLEMENTATION DEFINED fault (Unsupported Exclusive or Atomic access)",

        0b111101 => "Section Domain Fault",
        0b111110 => "Page Domain Fault",

        _ => "Reserved / Unknown",
    }
}

fn handle_svc(frame: &mut TrapFrame) -> usize {
    match frame.x8 {
        0 => {
            debug!("Program exited!");
            set_return_to_kernel_loop();
            0
        }
        67 => {
            let response = mailbox::read_soc_temp([0]).unwrap();
            response[1] as usize
        }
        _ => 0,
    }
}

fn log_sync_exception() {
    let source_el = read_exception_source_el() >> 2;
    debug!("--------Sync Exception in EL{}--------", source_el);
    debug!("Exception escalated to EL {}", get_current_el());
    debug!("Current EL: {}", get_current_el());
    let esr: EsrElX = EsrElX::from(read_esr_el1());
    debug!("{:?}", esr);
    debug!("Return address: {:#x}", read_elr_el1());
    debug!("-------------------------------------");
}
