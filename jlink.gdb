# print demangled symbols by default
set print asm-demangle on

# JLink
target extended-remote :2331
monitor semihosting enable
monitor semihosting IOClient 3

monitor reset
load
