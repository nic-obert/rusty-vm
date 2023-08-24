# to_signed


.text:

    # Convert a positive integer to its two's complement negative counterpart
    # Args:
    #   - num: the number to converter
    #
    # Return:
    #   - r1: the converted number
    #
    %% to_signed num:

        mov8 r2 {num}
        mov1 r1 0
        isub
    
    %endmacro

