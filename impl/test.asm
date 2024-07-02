
.include:

    "stdio/print.asm"
    "math.asm"
    "archlib.asm"


.data:

    ERROR_POW string "Invalid input for powi function.\0"


.text:

@start:

    
    mov8 r1 4
    call expi

    cmp1 error =NO_ERROR
    jmpz no_error

        !println_str ERROR_POW

    @no_error

    !println_float r1

