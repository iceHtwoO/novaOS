
.global vector_table
.extern irq_handler

.macro    ventry    label
.align    7
b    \label
.endm

.section .vector_table, "ax"
vector_table:
    ventry .
    ventry .
    ventry .
    ventry .

    ventry synchronous_interrupt_no_el_change  // Synchronous Exception  0x200
    ventry irq_handler                // IRQ(Interrupt Request) 0x280
    ventry .
    ventry .

    ventry synchronous_interrupt_imm_lower_aarch64
    ventry irq_handler


.align 4
.global el2_to_el1
el2_to_el1:

    mov x0, #(1 << 31)
    msr HCR_EL2, x0

    // Set SPSR_EL2: return to EL1h
    mov x0, #(0b0101)
    msr SPSR_EL2, x0

    // Set return address to kernel_main
    ldr x0, =kernel_main
    msr ELR_EL2, x0

    // Set SP_EL1 to stack base
    ldr x0, =__stack_end
    msr SP_EL1, x0

    // Set VBAR_EL1 to vector table
    adr x0, vector_table
    msr VBAR_EL1, x0

    // Disable MMU
    ldr x0, =SCTLR_EL1_CONF
    msr sctlr_el1, x0

    // SIMD should not be trapped
    mrs x0, CPACR_EL1
    mov x1, #(0b11<<20)
    orr x0,x0, x1
    msr CPACR_EL1,x0

    // Return to EL1
    eret

.align 4
.global el1_to_el0
el1_to_el0:

    // Set SPSR_EL1: return to EL0t
    mov x0, #(0b0000)
    msr SPSR_EL1, x0

    // Set return address to el0
    ldr x0, =el0
    msr ELR_EL1, x0

    // Set SP_EL1 to stack base
    ldr x0, =__stack_end_el0
    msr SP_EL0, x0

    // Return to EL0
    eret


.align 4
irq_handler:
    sub sp, sp, #176
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x29, [sp, #144]
    stp x30, xzr, [sp, #160]

    bl rust_irq_handler

    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x29, [sp, #144]
    ldp x30, xzr, [sp, #160]
    add sp, sp, #176

    eret

.align 4
synchronous_interrupt_imm_lower_aarch64:
    sub sp, sp, #176
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x29, [sp, #144]
    stp x30, xzr, [sp, #160]

    bl rust_synchronous_interrupt_imm_lower_aarch64

    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x29, [sp, #144]
    ldp x30, xzr, [sp, #160]
    add sp, sp, #176

    eret

.align 4
synchronous_interrupt_no_el_change:
    sub sp, sp, #176
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x29, [sp, #144]
    stp x30, xzr, [sp, #160]

    bl rust_synchronous_interrupt_no_el_change

    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x29, [sp, #144]
    ldp x30, xzr, [sp, #160]
    add sp, sp, #176

    eret
