
.include:

    string.asm

.data:

    S1 string "hel"
    S2 string "xxx\n\0"

.text:

@start

    mov8 r1 S1
    mov8 r2 S2
    mov8 r3 3

    call memmove

    mov8 print S2
    printstr
    
    
