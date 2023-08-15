
.include:

    string.asm

.data:

    S1 string "hellooooooooooooo\n\0"
    S2 string "xxxxxxxxxx\n\0"

.text:

@start

    mov8 r1 S1
    mov8 r2 S2
    mov8 r3 7

    call strncpy

    mov8 print S2
    printstr
    
    
