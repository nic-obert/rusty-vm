
.include:

    "foo.asm"
    "stdio.asm"

.text:

    %-ZERO: 10
    %-r1_zero: r1 =ZERO

    mov1 =r1_zero

    !println_uint r1

