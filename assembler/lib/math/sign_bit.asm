# sign_bit


.text:

    # Return the sign bit of a number
    # Args:
    #   - num: the number
    #
    # Return:
    #   - r1: the sign bit of the number
    #
    %% sign_bit num:

        mov8 r1 {num}
        mov1 r2 63
        shr

    %endmacro

