# print
# Export useful macros for printing to the console


.include:

    "archlib.asm"
    "asmutils/functional.asm"


.text:

    %% println_char c:

        mov1 print {c}
        mov1 int =PRINT_CHAR
        intr
        !println

    %endmacro


    %% print_char c:

        mov1 print {c}
        mov1 int =PRINT_CHAR
        intr

    %endmacro


    %% print_char_reg reg:

        mov print {reg}
        mov1 int =PRINT_CHAR
        intr

    %endmacro


    %% println_char_reg reg:

        mov print {reg}
        mov1 int =PRINT_CHAR
        intr
        !println

    %endmacro


    %% print_str s:

        mov8 print {s}
        mov1 int =PRINT_STRING
        intr
        
    
    %endmacro


    %% println_str s:

        mov8 print {s}
        mov1 int =PRINT_STRING
        intr
        !println

    %endmacro


    %% print_bytes addr len:

        !save_reg_state r1

        mov8 r1 {len}
        mov8 print {addr}
        mov1 int =PRINT_BYTES
        intr

        !restore_reg_state r1

    %endmacro


    %% println_bytes addr len:

        !save_reg_state r1

        mov8 r1 {len}
        mov8 print {addr}
        mov1 int =PRINT_BYTES
        intr
        !println

        !restore_reg_state r1

    %endmacro


    %% print_int i:

        mov8 print {i}
        mov1 int =PRINT_SIGNED
        intr

    %endmacro


    %% println_int i:

        mov8 print {i}
        mov1 int =PRINT_SIGNED
        intr
        !println

    %endmacro


    %% print_uint i:

        mov8 print {i}
        mov1 int =PRINT_UNSIGNED
        intr

    %endmacro


    %% println_uint i:

        mov8 print {i}
        mov1 int =PRINT_UNSIGNED
        intr
        !println

    %endmacro


    %% print_float n:

        mov8 print {n}
        mov1 int =PRINT_FLOAT
        intr
    
    %endmacro


    %% println_float n:

        mov8 print {n}
        mov1 int =PRINT_FLOAT
        intr
        !println
    
    %endmacro


    %% println:

        mov1 print 10
        mov1 int =PRINT_CHAR
        intr

    %endmacro


    %% flush_stdout:

        mov1 int =FLUSH_STDOUT
        intr

    %endmacro

