.include:

    "stdio.asm"

.data:

    @msg
    dcs "hello world"

.text:

    !println_str msg

    exit
