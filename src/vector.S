.section .vector_table  , "ax"
.extern irq_handler

.macro    ventry    label
.align    11
b    \label
.endm

.global vector_table
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
    ventry .
    ventry .

    ventry .
    ventry .
    ventry .
    ventry .
