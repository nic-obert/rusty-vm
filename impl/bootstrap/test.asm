.include:

    "stdio.asm"
    "archlib.asm"
    "string.asm"


.data:

    @line
    ds "  \0store \0 v1 4\0"

.text:

    mov8 r1 line
    call strtok

    # Calculate the size of the first token
    mov r3 r1
    mov r1 r2
    mov r2 r3
    isub

    !println_uint r1

    exit
