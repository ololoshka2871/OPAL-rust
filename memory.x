MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  /* These values correspond to the LM3S6965, one of the few devices QEMU can emulate */

  /* 3.2 FLASH main features: page size = 2K */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K - 3 * 2K
  WRITER_TEST_AREA: ORIGIN = 0x08000000 + 256K - 3 * 2K, LENGTH = 2 * 2K
  SETTINGS: ORIGIN = 0x08000000 + 256K - 2K, LENGTH = 2K

  RAM : ORIGIN = 0x20000000, LENGTH = 64K
  RAM2 : ORIGIN = 0x10000000, LENGTH = 0x4000
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* stack -> RAM2 */
_stack_start = ORIGIN(RAM2) + LENGTH(RAM2);

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */

/* Example of putting non-initialized variables into custom RAM locations. */
/* This assumes you have defined a region RAM2 above, and in the Rust
   sources added the attribute `#[link_section = ".ram2bss"]` to the data
   you want to place there. */
/* Note that the section will not be zero-initialized by the runtime! */
/* SECTIONS {
     .ram2bss (NOLOAD) : ALIGN(4) {
       *(.ram2bss);
       . = ALIGN(4);
     } > RAM2
   } INSERT AFTER .bss;
*/

SECTIONS {
   .writer_test_area (NOLOAD): ALIGN(8)
   {
      __writer_area = .;
      KEEP(*(.writer_test_area*));
   } > WRITER_TEST_AREA

   .settings (NOLOAD) : ALIGN(8)
   {
      __settings_pos = .;
      KEEP(*(.settings*));
   } > SETTINGS
}
