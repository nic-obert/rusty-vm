# print
# Export useful macros for printing to the console


.text:

    %% print_string_ln reg:

        mov print {reg}
        printstr
        mov1 print 10
        printc

    %endmacro


    %% print_string reg:

        mov print {reg}
        printstr

    %endmacro


    %% print_int_ln reg:

        mov print {reg}
        printi
        mov1 print 10
        printc
    
    %endmacro


    %% print_uint_ln reg:

        mov print {reg}
        printu
        mov1 print 10
        printc
    
    %endmacro

