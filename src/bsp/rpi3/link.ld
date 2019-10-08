ENTRY(_start)

SECTIONS
{
    /* Set current address to the value from which the RPi4 starts execution */
    . = 0x80000 - 0x1000;

    __code = .;
    .text ALIGN(8):
    {
        *(.text._start) *(.text*)
    }

    .rodata ALIGN(8) :
    {
        *(.rodata .rodata.*) *(.got)
    }

    .data ALIGN(8):
    {
        *(.data .data.*)
    }

    /*Align to 8 byte boundary */
    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss);
        __bss_end = .;
    }
    __end = .;

    /DISCARD/ : { *(.comment*) }
}