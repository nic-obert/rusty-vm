.include:

    "stdio.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "asmutils/error_handling.asm"


.data:



.text:

    %- INPUT_BUF_SIZE: 16

    pushsp8 =INPUT_BUF_SIZE
    mov r8 stp

    # get string input
    mov r1 r8
    mov8 r2 =INPUT_BUF_SIZE
    call read_word
    !println_str stp
    call flush_stdin

    call read_word
    !println_str stp


    exit
