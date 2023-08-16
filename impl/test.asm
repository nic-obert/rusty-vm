
.include:

    string.asm


.text:

@start

    mov8 r1 S1
    mov8 r2 S2
    mov8 r3 5

    call memmove

    mov8 print S2
    printstr
    
    exit


.data:

    S1 string "abcde"
    S2 string "fghijklmnopq\n\0"

