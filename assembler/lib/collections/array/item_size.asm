

.text:

    # Return the item size of the array
    #
    # Args:
    #   - array: array address (8 bytes)
    #
    # Return:
    #   - r1: item size (8 bytes)
    #
    %% array_item_size array:

        # Get the item_size field
        mov8 r1 [{array}]

    %endmacro

