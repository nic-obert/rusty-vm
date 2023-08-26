.include:

    asmutils/load_arg.asm

    item_size.asm


.text:

    # Set the array item at `index` to `value`
    #
    # Args:
    #   - array: array address (8 bytes)
    #   - index: item index to set (8 bytes)
    #   - value: the address of the value to set (8 bytes)
    #
    %% array_set array index value:

        push8 {array}
        push8 {index}
        push8 {value}

        call array_set

        popsp1 24

    %endmacro

    @@ array_set


        ret

