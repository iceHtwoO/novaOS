SECTIONS {
    . = 0x80000;

    .text ALIGN(4) : {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data ALIGN(2M) : {
        _data = .;
        *(.data .data.*)
    }

    .bss (NOLOAD) : {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    .vector_table ALIGN(2048) : {
        KEEP(*(.vector_table))
    }

    .translation_table_l1 ALIGN(4096) : {
        __translation_table_l1_start = .;
        . += 4096;
        __translation_table_l1_end = .;
    }

    .translation_table_l2 ALIGN(4096) : {
        __translation_table_l2_start = .;
        . += 4096;
        __translation_table_l2_end = .;
    }

    .heap : ALIGN(16)
    {
    __heap_start = .;
    . += 100K; #100kB
    __heap_end = .;
    }

    .stack : ALIGN(16)
    {
    __stack_start = .;
    . += 10K; #10kB stack
    __stack_end = .;
    }

    .stack_el0 : ALIGN(2M)
    {
    __stack_start_el0 = .;
    . += 10K; #10kB stack
    __stack_end_el0 = .;
    }

    _end = .;
}

__bss_size = (__bss_end - __bss_start) >> 3;
