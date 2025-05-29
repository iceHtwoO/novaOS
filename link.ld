SECTIONS {
    . = 0x80000;

    .text ALIGN(4) : {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
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

    .stack 0x8008000 : ALIGN(16)
    {
    __stack_start = .;
    .+=0x10000;
    __stack_end = .;
    }


    _end = .;
}

__bss_size = (__bss_end - __bss_start) >> 3;
