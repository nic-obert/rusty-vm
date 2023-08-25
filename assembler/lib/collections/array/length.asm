

.text:

    # Return the length of the array
    #
    # Args:
    #   - array: array address (8 bytes)
    #
    # Return:
    #   - r1: array length (8 bytes)
    #
    %% array_length array:

        # Save the register states
        push8 r2

        # Calculate the address of the length field
        mov8 r1 {array}
        mov1 r2 8
        iadd

        # Get the length field
        mov8 r1 [r1]

        # Restore the register states
        pop8 r2

    %endmacro

