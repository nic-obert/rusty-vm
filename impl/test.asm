
.include:

    "stdio/print.asm"
    "archlib.asm"


.data:

    @ERROR_POW
    ds "Invalid input for powi function.\0"

    @TEST_NUM
    dn 8 80


.text:

@start


    !println_str ERROR_POW
    !println_int [TEST_NUM]


