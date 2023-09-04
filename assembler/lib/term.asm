# term
# Terminal-related macros and definitions to access the terminal module via interrupts


.include:

    @@ interrupts.asm


.text:

    # Terminal-specific operation codes

    %%- TERM_GOTO: 0
    %%- TERM_CLEAR: 1
    %%- TERM_BLINK: 2
    %%- TERM_BOLD: 3
    %%- TERM_UNDERLINE: 4
    %%- TERM_RESET: 5
    %%- TERM_HIDE_CURSOR: 6
    %%- TERM_SHOW_CURSOR: 7
    %%- TERM_DOWN: 8
    %%- TERM_UP: 9
    %%- TERM_RIGHT: 10
    %%- TERM_LEFT: 11
    %%- TERM_BLINKING_BLOCK: 12
    %%- TERM_STEADY_BLOCK: 13
    %%- TERM_BLINKING_UNDERLINE: 14
    %%- TERM_STEADY_UNDERLINE: 15
    %%- TERM_BLINKING_BAR: 16
    %%- TERM_STEADY_BAR: 17
    %%- TERM_SAVE_CURSOR_POSITION: 18
    %%- TERM_RESTORE_CURSOR_POSITION: 19
    %%- TERM_CLEAR_LINE: 20
    %%- TERM_CLEAR_AFTER: 21
    %%- TERM_CLEAR_BEFORE: 22
    %%- TERM_CLEAR_UNTIL_NEWLINE: 23
    %%- TERM_GET_TERMINAL_SIZE: 24
    %%- TERM_GET_TERMINAL_SIZE_PIXELS: 25
    %%- TERM_GET_CURSOR_POSITION: 26

    # Constants

    %- TERM_CODE_REG: print


    %% term_goto:

        mov1 =TERM_CODE_REG =TERM_GOTO
        intr =TERM_INTR
    
    %endmacro


    %% term_clear:

        mov1 =TERM_CODE_REG =TERM_CLEAR
        intr =TERM_INTR
    
    %endmacro


    %% term_blink:

        mov1 =TERM_CODE_REG =TERM_BLINK
        intr =TERM_INTR
    
    %endmacro


    %% term_bold:

        mov1 =TERM_CODE_REG =TERM_BOLD
        intr =TERM_INTR
    
    %endmacro


    %% term_underline:

        mov1 =TERM_CODE_REG =TERM_UNDERLINE
        intr =TERM_INTR
    
    %endmacro


    %% term_reset:

        mov1 =TERM_CODE_REG =TERM_RESET
        intr =TERM_INTR
    
    %endmacro


    %% term_hide_cursor:

        mov1 =TERM_CODE_REG =TERM_HIDE_CURSOR
        intr =TERM_INTR
    
    %endmacro


    %% term_show_cursor:

        mov1 =TERM_CODE_REG =TERM_SHOW_CURSOR
        intr =TERM_INTR
    
    %endmacro


    %% term_down:

        mov1 =TERM_CODE_REG =TERM_DOWN
        intr =TERM_INTR
    
    %endmacro


    %% term_up:

        mov1 =TERM_CODE_REG =TERM_UP
        intr =TERM_INTR
    
    %endmacro


    %% term_right:

        mov1 =TERM_CODE_REG =TERM_RIGHT
        intr =TERM_INTR
    
    %endmacro


    %% term_left:

        mov1 =TERM_CODE_REG =TERM_LEFT
        intr =TERM_INTR
    
    %endmacro


    %% term_blinking_block:

        mov1 =TERM_CODE_REG =TERM_BLINKING_BLOCK
        intr =TERM_INTR
    
    %endmacro


    %% term_steady_block:

        mov1 =TERM_CODE_REG =TERM_STEADY_BLOCK
        intr =TERM_INTR
    
    %endmacro


    %% term_blinking_underline:

        mov1 =TERM_CODE_REG =TERM_BLINKING_UNDERLINE
        intr =TERM_INTR
    
    %endmacro


    %% term_steady_underline:

        mov1 =TERM_CODE_REG =TERM_STEADY_UNDERLINE
        intr =TERM_INTR
    
    %endmacro


    %% term_blinking_bar:

        mov1 =TERM_CODE_REG =TERM_BLINKING_BAR
        intr =TERM_INTR
    
    %endmacro


    %% term_steady_bar:

        mov1 =TERM_CODE_REG =TERM_STEADY_BAR
        intr =TERM_INTR
    
    %endmacro


    %% term_save_cursor_position:

        mov1 =TERM_CODE_REG =TERM_SAVE_CURSOR_POSITION
        intr =TERM_INTR
    
    %endmacro


    %% term_restore_cursor_position:

        mov1 =TERM_CODE_REG =TERM_RESTORE_CURSOR_POSITION
        intr =TERM_INTR
    
    %endmacro


    %% term_clear_line:

        mov1 =TERM_CODE_REG =TERM_CLEAR_LINE
        intr =TERM_INTR
    
    %endmacro


    %% term_clear_after:

        mov1 =TERM_CODE_REG =TERM_CLEAR_AFTER
        intr =TERM_INTR
    
    %endmacro


    %% term_clear_before:

        mov1 =TERM_CODE_REG =TERM_CLEAR_BEFORE
        intr =TERM_INTR
    
    %endmacro


    %% term_clear_until_newline:

        mov1 =TERM_CODE_REG =TERM_CLEAR_UNTIL_NEWLINE
        intr =TERM_INTR
    
    %endmacro


    %% term_get_terminal_size:

        mov1 =TERM_CODE_REG =TERM_GET_TERMINAL_SIZE
        intr =TERM_INTR
    
    %endmacro


    %% term_get_terminal_size_pixels:

        mov1 =TERM_CODE_REG =TERM_GET_TERMINAL_SIZE_PIXELS
        intr =TERM_INTR
    
    %endmacro


    %% term_get_cursor_position:

        mov1 =TERM_CODE_REG =TERM_GET_CURSOR_POSITION
        intr =TERM_INTR
    
    %endmacro

