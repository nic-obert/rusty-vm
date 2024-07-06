.include:

    "stdio.asm"


.data:

    @strlen
    offsetfrom str
    @str
    ds "hello"

.text:
    
    !println_bytes str strlen
    !println_uint strlen
