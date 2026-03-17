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

    .bss ALIGN(16) (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    }

    .vector_table ALIGN(2K) : {
        KEEP(*(.vector_table))
    }

    # EL2 Stack
    .stack ALIGN(16): {
    __stack_start = .;
    . += 100K; #100kB stack
    . = ALIGN(16);
    __stack_end = .;
    }

    . = ALIGN(2M);

    __kernel_end = .;

    .stack_el0 : {
    __stack_start_el0 = .;
    . += 10K; #10kB stack
    __stack_end_el0 = .;
    }

    . = ALIGN(2M);
    _end = .;
}

__bss_size = (__bss_end - __bss_start) >> 3;
