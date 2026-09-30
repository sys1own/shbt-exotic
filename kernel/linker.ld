/*
 * linker.ld — GNU linker script for the shbt-exotic bare-metal microkernel.
 *
 * Pre-allocates the 2,112-byte contiguous .stinespring_frame SRAM arena on a
 * 64-byte boundary, partitioned at runtime into a 640 B active residual
 * (eta_A = 10/33) and a 1,472 B dark ledger (eta_D = 23/33, 124 Fibonacci
 * braid descriptors).
 */

ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 0x00100000
}

SECTIONS
{
    . = ORIGIN(RAM);

    .text : {
        *(.text.init)
        *(.text .text.*)
        *(.gnu.linkonce.t.*)
    } > RAM

    .rodata : {
        *(.rodata .rodata.*)
        *(.gnu.linkonce.r.*)
    } > RAM

    .data : {
        __data_start = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.gnu.linkonce.d.*)
        __data_end = .;
    } > RAM

    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        *(.gnu.linkonce.b.*)
        __bss_end = .;
    } > RAM

    /*
     * Stinespring frame arena: 2112 B, 64-byte aligned, NOLOAD.
     * Partition symbols:
     *   [__stinespring_start, __active_end)        640 B  active residual
     *   [__active_end, __stinespring_end)         1472 B  dark ledger
     */
    .stinespring_frame (NOLOAD) : ALIGN(64) {
        __stinespring_start = .;
        . = . + 640;
        __active_end = .;
        . = . + 1472;
        __stinespring_end = .;
    } > RAM

    __end = .;
}
