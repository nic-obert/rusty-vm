.include:

    "stdio.asm"


.data:




.text:

    mov8 r1 0x000000000000000F
    swpe
    !println_uint r1

    mov8 r2 0x0F00000000000000
    !println_uint r2


    cmp r1 r2
    jmpz equal
        printstr "Numbers are not equal\n"
        jmp after
    @equal
        printstr "Numbers are equal\n"
    @after

    exit
