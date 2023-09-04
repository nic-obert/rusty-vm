# stdbool
# Definitions and functions for boolean logic


.text:

    # Convert an integer into a boolean
    #
    # Args:
    #   - reg: register where the integer to convert is stored
    #
    # Return:
    #   - r1: the corresponding boolean value
    #
    %% bool_from_int_reg reg:

        # Convert number to boolean
        cmp1 {reg} 0
        # Invert the boolean
        cmp1 zf 0
        # Return the value
        mov r1 zf

    %endmacro


    # Invert a boolean value in-place
    #
    # Args:
    #   - reg: the register where the boolean value is stored
    #
    %% bool_invert_reg reg:

        # cmp x 0 -> 1 if x == 0, 0 if x != 0
        cmp1 {reg} 0
        mov {reg} zf

    %endmacro


    # Invert the boolean value in the zf register.
    # Assumes zf contains a canonical boolean value.
    #
    %% bool_invert_zf:

        cmp1 zf 0

    %endmacro

