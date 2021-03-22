/* FLASH actually starts at 0x08000000, but we want to load everything into RAM.
 * 1K of working RAM is enough.
 */
MEMORY {
    FLASH : ORIGIN = 0x20000000, LENGTH = 19K
    RAM   : ORIGIN = 0x20004c00, LENGTH =  1K
}