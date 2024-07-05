
.include:

    "stdio/print.asm"
    "archlib.asm"


.data:

    @MSG
    ds 'Current position: 
    is not me
    but mine is bigger\0"

    @TEST_NUM
    dn 8 80


.text:

@start


    !print_str MSG

    jmp after
    dn 8 12
    @after

    mov8 r1 $
    #!println_uint [r1]

    mov8 print r1
    mov1 int =PRINT_UNSIGNED
    intr


