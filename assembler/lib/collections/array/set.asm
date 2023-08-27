.include:

    asmutils/functional.asm
    string/memcpy.asm

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

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r5
        !save_reg_state r6
        !save_reg_state r7
        !save_reg_state r8

        %- array: r8
        %- item_size: r7
        %- index: r6
        %- value: r5

        !load_arg8 8 =value
        !load_arg8 16 =index
        !load_arg8 24 =array

        !array_item_size =array
        mov =item_size r1

        # Calculate the item offset (index * item_size)
        # r1 is already `item_size`
        mov r2 =index
        imul

        # Calculate the item address (array* + offset)
        mov r2 =array
        iadd

        # Copy the value into the array slot
        !memcpy =value r1 =item_size


        !restore_reg_state r8
        !restore_reg_state r7
        !restore_reg_state r6
        !restore_reg_state r5
        !restore_reg_state r2
        !restore_reg_state r1

        ret

