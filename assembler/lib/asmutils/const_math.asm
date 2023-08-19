# const_math
# Export macros to perform math with a constant argument


.text:

    %% iadd_const_reg reg amount:

        mov r1 {reg}
        mov8 r2 {amount}
        iadd

        mov {reg} r1
    
    %endmacro


    %% isub_const_reg reg amount:

        mov r1 {reg}
        mov8 r2 {amount}
        isub

        mov {reg} r1
    
    %endmacro

    