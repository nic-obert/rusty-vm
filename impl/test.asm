.include:

    "stdio.asm"


.data:

    @foo
    da [u8:1] [ [107], [98], [105], [105], [110], [0] ]

.text:

    !println_str foo

