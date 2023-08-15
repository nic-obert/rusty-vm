
.include:

    stdlib.asm

.data:

    S string "-301\0"

.text:

@start

    mov8 r1 S

    call ascii_to_int

    mov print r1
    printu
    mov1 print 10
    printc
    mov print r1
    printi
    mov1 print 10
    printc
    
