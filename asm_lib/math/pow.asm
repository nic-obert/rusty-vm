# pow
# Compute powers

.include:

    "archlib.asm"
    "asmutils/functional.asm"


.text:

    # Args:
    #   - r1: integer exponent < 63 (unchecked)
    #
    # Return:
    #   - r1: the integer result
    #
    @@ pow2

        !save_reg_state r2

        mov r2 r1
        mov8 r1 2
        shl

        !restore_reg_state r2

        ret


    # Elevate `base` to the `exp` power
    # Args:
    #   - r1: integer base
    #   - r2: integer exponent
    #
    # Return:
    #   - r1: the integer result
    #
    @@ powi

        !save_reg_state r3
        !save_reg_state r4

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

        !restore_reg_state r4
        !restore_reg_state r3

        ret


    @ negative_exp

        # Reciprocals are usually not integers.
        # 1/5^4, for example, does not result in an integer.
        # While this is ok in a floating point power, the result of this function must be an integer.

        mov1 error =INVALID_INPUT

        !restore_reg_state r4
        !restore_reg_state r3

        ret


    @ zero_exp

        # x^0 = 1
        mov1 r1 0

        !restore_reg_state r4
        !restore_reg_state r3

        ret
