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
    %- modifier: r4
    %- key: r5
    %- x: r6
    %- y: r7
    %- POINTER_CHAR: '#'

    call setup_screen

    # Initialize coordinates to the screen center
    mov8 r1 [COLUMNS]
    mov1 r2 2
    idiv
    mov =x r1

    mov8 r1 [ROWS]
    idiv
    mov =y r1

    # Allocate the keyboard listener buffer on the stack
    pushsp1 =TERM_KEY_DATA_SIZE
    mov r1 stp
    
    !term_get_key_listener
    mov =listener r1

    @mainloop

        !has_key_data =listener
        cmp1 r1 =TRUE
        jmpnz mainloop

            !read_key_data =listener

            cmp1 r1 =TERM_KEYCODE_ESC
            jmpz break_mainloop

            mov =modifier r1
            mov =key r2

            # Handle the case where the key is a character
            cmp1 =modifier =TERM_KEYCODE_CHAR
            jmpnz not_char

                cmp1 =key 'w'
                jmpnz not_w
                    dec =y
                @not_w
                cmp1 =key 's'
                jmpnz not_s
                    inc =y
                @not_s
                cmp1 =key 'a'
                jmpnz not_a
                    dec =x
                @not_a
                cmp1 =key 'd'
                jmpnz not_d
                    inc =x
                @not_d

                mov r1 =x
                mov r2 =y

            @not_char

            !term_clear

            !term_goto
            !term_bold
            !print_char =POINTER_CHAR
            !term_reset
            
            !flush_stdout

            jmp mainloop
    
    @break_mainloop


    !term_stop_key_listener

    call restore_screen

    exit

