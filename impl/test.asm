.include:

    "stdio.asm"

.data:

    @s
    dcs "hello"

.text:

    !println_str s

   exit
