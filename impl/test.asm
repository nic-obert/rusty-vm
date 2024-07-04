
.include:

    "stdio/print.asm"
    "archlib.asm"


.data:

    @ERROR_POW
    ds "Invalid input for powi function.\0"


.text:

@start


    !println_str ERROR_POW


