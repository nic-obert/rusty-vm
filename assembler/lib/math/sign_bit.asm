# sign_bit
# Get the sign bit of the given number and store it in r1


.text:

    %% sign_bit num:

        mov8 r1 {num}
        mov1 r2 63
        shr

    %endmacro

