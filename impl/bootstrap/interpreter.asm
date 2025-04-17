.include:

    "allocators/object_allocator.asm"
    "stdio.asm"
    "string.asm"


.data:


    @shell_prompt
    dcs "> "

    @exit_command
    dcs "exit"

    @print_string_command
    dcs "prints"

.text:

    # A shell-like interpreter
    #
    %- INPUT_BUF_SIZE: 256
    %- INPUT_BUF: r3
    %- TOKEN_START: r4
    %- TOKEN_END: r5
    %- RUNNING: r6
    %- TOKEN_LENGTH: r7

    # Object size
    #mov8 r1 8
    # Max objects
    #mov8 r2 4
    #call init_object_allocator

    # Allocate a buffer for the input string
    pushsp8 =INPUT_BUF_SIZE
    mov =INPUT_BUF stp

    mov1 =RUNNING 1

    @main_loop
        cmp1 =RUNNING 0
        jmpz terminate

        !print_str shell_prompt
        mov r1 =INPUT_BUF
        mov8 r2 =INPUT_BUF_SIZE
        call read_str

        call interpret_line

        jmp main_loop

    @terminate

    exit


@interpret_line


    # Tokenize first command
    mov r1 =INPUT_BUF
    call strtok
    mov =TOKEN_START r1
    mov =TOKEN_END r2
    # Calculate the token length
    mov r1 =TOKEN_END
    mov r2 =TOKEN_START
    isub
    mov =TOKEN_LENGTH r1

    # Exit

    !strncmp exit_command =TOKEN_START =TOKEN_LENGTH
    cmp1 r1 0
    mov =RUNNING zf

    # Print string
    !strncmp print_string_command =TOKEN_START =TOKEN_LENGTH
    cmp1 r1 1
    jmpnz no_print_string_command
        !println_str =TOKEN_END
    @no_print_string_command

    ret
