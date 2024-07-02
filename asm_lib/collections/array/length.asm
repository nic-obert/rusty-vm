.include:

    "metadata.asm"


.text:

    # Return the length of the array
    #
    # Args:
    #   - array: array address (8 bytes)
    #
    # Return:
    #   - r1: array length (8 bytes)
    #
    # Invalidate:
    #   - r2
    #
    %% array_length array:

        !array_get_length_ptr     

        # Get the length field
        mov8 r1 [r1]

    %endmacro

