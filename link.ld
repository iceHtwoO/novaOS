SECTIONS {
    . = 0x80000;

    .text ALIGN(4) : {
        KEEP(*(.text._start))
        *(.text .text.*)
    }
    .vector_table ALIGN(2K) : {
        KEEP(*(.vector_t))
    }

    . = ALIGN(4K);
    __text_end = .;

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data  : {
        *(.data .data.*)
    }

    .bss ALIGN(16) (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    }

    . = ALIGN(2M);

    __share_end = .;

    # EL2 Stack
    .stack ALIGN(16): {
    __stack_start = .;
    . += 100K; #100kB stack
    . = ALIGN(16);
    __stack_end = .;
    }

    . = ALIGN(2M);

    __kernel_end = .;
}

__bss_size = (__bss_end - __bss_start) >> 3;
