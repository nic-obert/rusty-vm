.include:

    asmutils/load_arg.asm
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
        popsp 16

    %endmacro

    @@ array_get

        # Save current register states
        push8 r1
        push8 r2
        push8 r5
        push8 r6
        push8 r7
        push8 r8


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


        # Restore previous register states
        pop8 r8
        pop8 r7
        pop8 r6
        pop8 r5
        pop8 r2
        pop8 r1

        ret

