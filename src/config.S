.section .text.config
.align 4
.global el2_to_el1
el2_to_el1:
    mov x0, #(1 << 31)
    msr HCR_EL2, x0

    // Set SPSR_EL2: return to EL1h
    mov x0, #(0b0101)
    msr SPSR_EL2, x0

    // Set return address to kernel_main
    adrp x0, KERNEL_VIRTUAL_MEM_SPACE
    ldr x1, [x0, :lo12:KERNEL_VIRTUAL_MEM_SPACE]

    adrp x0, kernel_main
    add  x0, x0, :lo12:kernel_main
    orr  x0, x0, x1
    msr ELR_EL2, x0

    // Set SP_EL1 to stack base
    adrp x0, EL1_STACK_TOP
    ldr  x1, [x0, :lo12:EL1_STACK_TOP]
    msr SP_EL1, x1

    // Set VBAR_EL1 to vector table
    adrp x0, vector_table
    add  x0, x0, :lo12:vector_table
    msr VBAR_EL1, x0

    isb

    adrp x0, SCTLR_EL1_CONF
    ldr  x1, [x0, :lo12:SCTLR_EL1_CONF]
    msr SCTLR_EL1, x1

    isb

    // SIMD should not be trapped
    mrs x0, CPACR_EL1
    mov x1, #(0b11<<20)
    orr x0,x0, x1
    msr CPACR_EL1,x0

    isb

    // Return to EL1
    eret

.section .text.config
.align 4
.global configure_mmu_el1
configure_mmu_el1:
    // Configure MMU
    adrp x0, TCR_EL1_CONF
    ldr  x1, [x0, :lo12:TCR_EL1_CONF]
    msr TCR_EL1, x1
    isb

    // MAIR0: Normal Mem.
    // MAIR1: Device Mem.
    mov x0, #0x04FF
    msr MAIR_EL1, x0
    isb

    // Configure translation table
    adrp x0, TRANSLATIONTABLE_TTBR0
    add  x1, x0, :lo12:TRANSLATIONTABLE_TTBR0
    msr TTBR0_EL1, x1

    adrp x0, TRANSLATIONTABLE_TTBR1
    add  x1, x0, :lo12:TRANSLATIONTABLE_TTBR1
    msr TTBR1_EL1, x1

    tlbi vmalle1
    dsb ish
    isb

    ret

.align 4
.global el1_to_el0
el1_to_el0:
    mov x8, x0
    // Set SPSR_EL1: return to EL0t
    mov x0, #(0b0000)
    msr SPSR_EL1, x0

    // Set return address to el0
    ldr x0, =el0
    msr ELR_EL1, x0

    // Set SP_EL1 to stack base
    adrp x0, EL0_STACK_TOP
    ldr  x1, [x0, :lo12:EL0_STACK_TOP]
    msr SP_EL0, x1

    isb

    mov x0, x8
    // Return to EL0
    eret
