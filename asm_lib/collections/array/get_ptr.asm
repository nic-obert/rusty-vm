.include:

    "asmutils/functional.asm"

    "item_size.asm"


.text:

    # Return a pointer to the array item at `index`
    #
    # Args:
    #   - array: array addres (8 bytes)
    #   - index: item index (8 bytes)
    #
    # Return:
    #   - r1: pointer to array[index]
    #
    %% array_get_ptr array index:

        push8 {array}
        push8 {index}

        call array_get_ptr

        popsp 16

    %endmacro

    @@ array_get_ptr

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- array: r3
        %- item_offset: r4

        # Load the index into r2 for later use
        !load_arg8 8 r2
        !load_arg8 16 =array

        !array_item_size =array

        # Calculate the item offset
        # r1 is `item_size`
        # r2 is `index`
        imul
        mov =item_offset r1

        # Calculate the item address (array* + data offset + item offset)
        !array_get_data_ptr
        mov r2 =item_offset
        iadd


        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret

