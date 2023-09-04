
.include:

    stdio/print.asm
    stdlib.asm


.data:

    UPPER u8 28


.text:

@start

    !rand_range 0 [UPPER]

    !println_uint r1

    exit

