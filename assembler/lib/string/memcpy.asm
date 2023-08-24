# memcpy


.include:

    asmutils/load_arg.asm


.text:

    # Copy `num` bytes from `src` into `dest` without checking for overlapping memory regions.
    # For copying overlapping memory regions, consider using `memmove`
    #
    # Args:
    #   - src: source memory region address (8 bytes)
    #   - dest: destination memory region address (8 bytes)
    #   - num: number of bytes to copy (8 bytes)
    #
    %% memcpy src dest num:

        push8 {src}
        push8 {dest}
        push8 {num}

        call memcpy

        popsp 24

    %endmacro

    @@ memcpy

        # Save current register states
        push8 r1
        push8 r2
        push8 r3


        %- src: r1
        %- dest: r2
        %- num: r3

        !load_arg8: 8 =num
        !load_arg8: 16 =dest
        !load_arg8: 24 =src

        @ loop

            # Check if all bytes have been copied
            cmp1 =num 0
            jmpz endloop

            # Copy the byte
            mov1 [=dest] [=src]

            # Increment the byte pointers
            inc =src
            inc =dest

            # Decrement the bytes still to write
            dec =num

            jmp loop

        @ endloop


        # Restore previous register states
        pop8 r3
        pop8 r2
        pop8 r1

        ret

