.include:

    asmutils/functional.asm
    string/memcpy.asm

    item_size.asm


.text:

    # Get the item in the array and push it on the stack
    #
    # Args:
    #   - array: array address (8 bytes)
    #   - index: item index to get (8 bytes)
    #
    # Return:
    #   - item on the stack: (item_size bytes)
    #
    %% array_get array index:

        # Allocate space on the stack to store the returned item
        !array_item_size {array}
        pushsp r1

        # Push arguments
        push8 {array}
        push8 {index}

        call array_get

        # Pop arguments
        popsp1 16

    %endmacro

    @@ array_get

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r5
        !save_reg_state r6
        !save_reg_state r7
        !save_reg_state r8

        %- item_addr: r5
        %- item_size: r6
        %- array: r7
        %- index: r8

        !load_arg8 8 =index
        !load_arg8 16 =array

        !array_item_size =array
        mov =item_size r1

        # Calculate the item offset (index * item_size)
        mov r2 =index
        imul

        # Caluclate the item address (array* + offset)
        mov r2 =array
        iadd
        mov =item_addr r1

        # Calculate the return value address
        mov r1 sbp
        mov1 r2 24
        isub

        # Copy the return value onto the stack
        !memcpy =item_addr r1 =item_size


        !restore_reg_state r8
        !restore_reg_state r7
        !restore_reg_state r6
        !restore_reg_state r5
        !restore_reg_state r2
        !restore_reg_state r1

        ret

