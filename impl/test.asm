
.include:

    stdlib.asm

.data:

    INVAID_BYTE string "The byte is invalid\n\0"

.text:

@start

    mov1 r1 'u'

    call ascii_to_digit

    cmp1 error INVALID_INPUT
    jmpz ok

    mov8 print INVAID_BYTE
    printstr
    exit

    @ok

    mov print r1
    printu
    mov1 print 10
    printc

    exit
    

