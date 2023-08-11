
.include:

    strlen.asm

.data:

    STR string "ciao\0"
    OUTPUT string "Number of chars: \0"

.text:

@start

    mov8 r1 STR
    call strlen

    mov8 print OUTPUT
    printstr

    mov print r1
    printu

    mov1 print 10
    printc



    

