
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

    ventry synchronous_interrupt      // Synchronous Exception  0x200
    ventry irq_handler                // IRQ(Interrupt Request) 0x280
    ventry .
    ventry .


.align 4
.global el2_to_el1
el2_to_el1:

    mov x0, #(1 << 31)
    msr HCR_EL2, x0

    // Set SPSR_EL2: return to EL1h
    mov x0, #(0b0101)
    msr SPSR_EL2, x0

    // Set return address to ELR_EL2
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

    isb

    // Return to EL1
    eret
