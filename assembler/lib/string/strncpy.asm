# strncpy


.include:

    asmutils/load_arg.asm


.text:

    # Copy `num` characters from `src` over to `dest`.
    # If the null termination is reached before writing `num` characters, 
    # `dest` is padded with zeros until `num` characters have been written.
    # The resulting string in `dest` shall not be considered null-terminated, 
    # as the null termination may not be copied.
    #
    # Args:
    #   - src: the source string address to copy from (8 bytes)
    #   - dest: the destination buffer address (8 bytes)
    #   - num: the number of characters to copy (8 bytes)
    #
    %% strncpy src dest num:

        push8 {src}
        push8 {dest}
        push8 {num}

        call strncpy

        popsp 24

    %endmacro

    @@ strncpy

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

        @ loop_copy

            # Check if the requested chars have been copied
            cmp1 =num 0
            jmpz endloop

            # Copy the chars
            mov1 [=dest] [=src]

            # Check if the source char is null (source string is finished)
            cmp1 [=src] 0
            jmpz pad_with_zeroes

            # Increment the char*
            inc =src
            inc =dest

            # Decrement the chars still to write
            dec =num

            jmp loop_copy


    @ pad_with_zeroes

        @ loop_pad

            # Check if the requested chars have been copied
            cmp1 =num 0
            jmpz endloop

            # Write 0 to destination
            mov1 [=dest] 0

            # Increment the char*
            inc =dest

            # Decrement the chars still to write
            dec =num

            jmp loop_pad


    @ endloop

    !restore_reg_state r5
    !restore_reg_state r4
    !restore_reg_state r3

    ret

