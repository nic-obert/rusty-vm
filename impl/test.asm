.include:

    "stdio.asm"
    "asmutils/functional.asm"



.data:


.text:

    mov8 r1 3
    mov8 r2 3
    cmp r1 r2

    jmpgr less


    exit


@less
    printstr "less\n"
    ret
