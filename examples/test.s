# clang --target=riscv32 test.s -nostdlib -o test.elf
.section .text
.global _start

_start:
print:
    li a0,1
    la a1,msg
    li a2,14
    li a7,64
    ecall
exit:
    li a0,0
    li a7,93
    ecall

.section .rodata
msg:
    .ascii "Hello, World!\n"
