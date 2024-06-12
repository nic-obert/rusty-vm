# pow
# Compute powers

.include:

    archlib.asm


.text:

    # Elevate `base` to the `exp` power
    # Args:
    #   - r1: integer base
    #   - r2: integer exponent
    #
    # Return:
    #   - r1: the integer result
    #
    @@ powi

        cmp8 r2 0
        jmplt negative_exp
        jmpz zero_exp

        # Here the exponent is positive and >0

        %- counter: r3
        %- base: r4

        mov =base r1
        mov =counter r2

        @ loop

            dec =counter
            jmpz endloop

            mov r2 =base
            imul

            jmp loop

        @endloop

        ret


    @ negative_exp

        # Reciprocals are usually not integers. 
        # 1/5^4, for example, does not result in an integer.
        # While this is ok in a floating point power, the result of this function must be an integer.

        mov1 error =INVALID_INPUT

        ret

    
    @ zero_exp

        # x^0 = 1
        mov1 r1 0

        ret