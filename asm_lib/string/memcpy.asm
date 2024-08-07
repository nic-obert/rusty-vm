# memcpy


.include:

    "asmutils/functional.asm"


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

        popsp1 24

    %endmacro

    @@ memcpy

        !set_fstart

        !save_reg_state r3
        !save_reg_state r4
        !save_reg_state r5

        %- src: r3
        %- dest: r4
        %- num: r5

        !load_arg8 8 =num
        !load_arg8 16 =dest
        !load_arg8 24 =src

        @ loop

            # Check if all bytes have been copied
            cmp8 =num 0
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
        !restore_reg_state r5
        !restore_reg_state r4
        !restore_reg_state r3

        ret

