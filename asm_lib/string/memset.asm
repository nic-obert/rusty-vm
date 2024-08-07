
.include:

    "asmutils/functional.asm"
    "stdio.asm"


.text:

    # Set `size` bytes of `buf` to `byte`
    #
    # Args:
    #   - buf: buffer pointer (8 bytes)
    #   - byte: fill byte (1 byte)
    #   - size: size of the buffer in bytes (8 bytes)
    #
    %% memset buf byte size:

        !println_uint stp

        push8 {size}
        push1 {byte}
        push8 {buf}

        !println_uint stp

        call memset

        popsp1 17

    %endmacro


    @@ memset

        #!set_fstart

        !println_uint stp

        #!save_reg_state r3
        #!save_reg_state r4
        #!save_reg_state r5

        %- dest: r3
        %- count: r4
        %- byte: r5

        pop8 r8

        !println_uint r8

        !println_uint stp

        #!load_arg8 8 =dest
        #!load_arg1 9 =byte
        #!load_arg8 17 =count

        pop8 =dest
        pop1 =byte
        pop8 =count

        !println_uint =dest
        !println_uint =byte
        !println_uint =count

        @loop

            cmp8 =count 0
            jmpz endloop

            mov1 [=dest] =byte

            inc =dest
            dec =count

            jmp loop

        @endloop

        #!restore_reg_state r5
        #!restore_reg_state r4
        #!restore_reg_state r3

        push r8

        ret


