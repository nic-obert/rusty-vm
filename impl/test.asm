
.include:

    string.asm
    stdio.asm
    stdlib.asm

.data:


.text:

@start

    mov1 r1 123

    call str_from_uint

    !print_char '"'
    !print_char '"'

    !println_str r1
    

    !free r1

