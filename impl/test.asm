.include:

    "stdio.asm"
    "asmutils/functional.asm"



.data:

    @a1
    db [97 98 99 100 101]

    @a2
    db [102 103 104 105 106]


.text:

    !println_bytes a1 5

    mov8 r1 a1
    mov8 r2 a2
    memcpyb1 5

    !println_bytes a1 5

    exit
