.include:

    "stdio.asm"
    "string.asm"


.data:

    @array1
    db [1 2 3 4]
    @array2
    db [5 6 7 8]


.text:

    !memset array1 0 4

    mov8 r1 array1
    mov1 r3 4
    @loop
    
        mov1 r2 [r1]
        !println_uint r2

        dec r3
        inc r1

        cmp1 r3 0
        jmpnz loop

    exit



    