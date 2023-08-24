# memswap


.include:

    string/memcpy.asm
    asmutils/load_arg.asm


.text:

    # Swap the content of two memory regions
    #
    # Args:
    #   - first: address of the first memory region (8 bytes)
    #   - second: address of the second memory region (8 bytes)
    #   - num: number of bytes to swap (8 bytes)
    #
    %% memswap first second num:

        push8 {first}
        push8 {second}
        push8 {num}

        call memswap

        popsp 24

    %endmacro

    @@ memswap

        push8 r4
        push8 r5
        push8 r6


        %- first: r4
        %- second: r5
        %- num: r6

        !load_arg8 8 =num
        !load_arg8 16 =second
        !load_arg8 24 =first

        # Allocate an intermediate buffer on the stack
        pushsp =num

        # Copy the first memory region into the buffer
        !memcpy =first sbp =num

        # Copy the second memory region into the first one
        !memcpy =second =first =num

        # Copy the temporary buffer into the second memory region
        !memcpy sbp =second =num

        # Pop the intermediate buffer from the stack
        popsp r6


        pop8 r6
        pop8 r5
        pop8 r4

        ret

