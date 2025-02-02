.include:

    "stdio.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "asmutils/error_handling.asm"


.data:



.text:

    mov1 int =INPUT_SIGNED
    intr
    !println_int input

    exit
