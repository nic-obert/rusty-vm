.include:

    term.asm
    time.asm
    stdio.asm
    stdlib.asm
    asmutils.asm
    stdbool.asm


.bss:

    COLUMNS u8
    ROWS u8


.text:

    # Constants

    %- TARGET_CHAR: 'X'


    # Functions

    @ setup_screen

        !term_hide_cursor
        !term_clear
        !flush_stdout

        !term_get_terminal_size
        mov8 [COLUMNS] r1
        mov8 [ROWS] r2

        !term_clear
        !flush_stdout

        ret


    @ restore_screen

        !term_reset
        !term_show_cursor

        ret


    @ spawn_target

        !rand_range 0 [COLUMNS]
        mov r3 r1
        !rand_range 0 [ROWS]
        mov r2 r1
        mov r1 r3

        !term_goto
        !print_char =TARGET_CHAR

        !flush_stdout

        ret
        

@start

    %- listener: r3

    call setup_screen

    pushsp1 =TERM_KEY_DATA_SIZE
    mov r1 sbp
    
    !term_get_key_listener
    mov =listener r1

    @loop

        !has_key_data =listener
        cmp1 r1 =TRUE
        jmpnz loop

            !read_key_data =listener

            cmp1 r1 =TERM_KEYCODE_ESC
            jmpnz loop


    call restore_screen

    exit

