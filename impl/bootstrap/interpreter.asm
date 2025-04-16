.include:

    "allocators/object_allocator.asm"
    "stdio.asm"
    "string.asm"


.data:


    @shell_prompt
    dcs "> "

    @exit_command
    dcs "exit"

    @print_command
    dcs "print"

.text:

    # A shell-like interpreter
    #
    %- INPUT_BUF: r3
    %- INPUT_BUF_SIZE: 256

    # Object size
    #mov8 r1 8
    # Max objects
    #mov8 r2 4
    #call init_object_allocator

    # Allocate a buffer for the input string
    pushsp8 =INPUT_BUF_SIZE
    mov =INPUT_BUF stp

    @main_loop

        !print_str shell_prompt
        mov r1 =INPUT_BUF
        mov8 r2 =INPUT_BUF_SIZE
        call read_str

        # Interpret the command
        # Keep the input buffer in r2
        mov r2 =INPUT_BUF

        # Exit
        mov8 r1 exit_command
        call strcmp
        cmp1 r1 1
        jmpz terminate

        # Print
        mov8 r1 print_command
        # todo need to strtok

        jmp main_loop

    @terminate

    exit
