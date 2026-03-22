use crate::{
    aarch64::registers::{daif::mask_all, read_elr_el1, read_esr_el1, read_exception_source_el},
    get_current_el,
    interrupt_handlers::{set_return_to_kernel_main, EsrElX, TrapFrame},
    pi3::mailbox,
    println,
};

/// Synchronous Exception Handler
///
/// Source is a lower Exception level, where the implemented level
/// immediately lower than the target level is using
/// AArch64.
#[no_mangle]
unsafe extern "C" fn rust_synchronous_interrupt_imm_lower_aarch64(frame: &mut TrapFrame) -> usize {
    mask_all();
    let esr: EsrElX = EsrElX::from(read_esr_el1());
    match esr.ec {
        0b100100 => {
            log_sync_exception();
            println!("Cause: Data Abort from a lower Exception level");
        }
        0b010101 => {
            println!("Cause: SVC instruction execution in AArch64");
            return handle_svc(frame);
        }
        _ => {
            println!("Unknown Error Code: {:b}", esr.ec);
        }
    }
    println!("Returning to kernel main...");

    set_return_to_kernel_main();
    0
}

fn handle_svc(frame: &mut TrapFrame) -> usize {
    match frame.x8 {
        67 => {
            let response = mailbox::read_soc_temp([0]).unwrap();
            response[1] as usize
        }
        _ => 0,
    }
}

fn log_sync_exception() {
    let source_el = read_exception_source_el() >> 2;
    println!("--------Sync Exception in EL{}--------", source_el);
    println!("Exception escalated to EL {}", get_current_el());
    println!("Current EL: {}", get_current_el());
    let esr: EsrElX = EsrElX::from(read_esr_el1());
    println!("{:?}", esr);
    println!("Return address: {:#x}", read_elr_el1());
    println!("-------------------------------------");
}
