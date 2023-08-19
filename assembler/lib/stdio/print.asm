# print
# Export useful macros for printing to the console


.include:

    @@ interrupts.asm


.text:

    %% print_char c:

        mov1 print {c}
        intr [PRINT_CHAR]

    %endmacro


    %% print_str s:

        mov8 print {s}
        intr [PRINT_STRING]
    
    %endmacro


    %% println_str s:

        mov8 print {s}
        intr [PRINT_STRING]
        mov1 print 10
        intr [PRINT_CHAR]

    %endmacro


    %% print_bytes addr:

        mov8 print {addr}
        intr [PRINT_BYTES]

    %endmacro


    %% print_int i:

        mov8 print {i}
        intr [PRINT_SIGNED]

    %endmacro


    %% print_uint i:

        mov8 print {i}
        intr [PRINT_UNSIGNED]

    %endmacro

