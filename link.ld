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

    .heap : ALIGN(16)
    {
    __heap_start = .;
    . += 0x10000; #10kB
    __heap_end = .;
    }

    .stack : ALIGN(16)
    {
    __stack_start = .;
    . += 0x10000; #10kB stack
    __stack_end = .;
    }

    .stack_el0 : ALIGN(16)
    {
    __stack_start_el0 = .;
    . += 0x10000; #10kB stack
    __stack_end_el0 = .;
    }


    _end = .;
}

__bss_size = (__bss_end - __bss_start) >> 3;
