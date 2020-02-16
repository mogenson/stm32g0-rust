/* STM32G031K8 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  SHARED : ORIGIN = 0x20000000, LENGTH = 4
  RAM : ORIGIN = 0x20000000 + 4, LENGTH = 8K - 4
}

_shared = ORIGIN(SHARED);
