
.include:

    "asmutils/functional.asm"


.text:

    # Set `size` bytes of `buf` to `byte`
    #
    # Args:
    #   - buf: buffer pointer (8 bytes)
    #   - byte: fill byte (1 byte)
    #   - size: size of the buffer in bytes (8 bytes)
    #
    %% memset buf byte size:

        push8 {size}
        push1 {byte}
        push8 {buf}

        call memset

        popsp1 17

    %endmacro


    @@ memset

        !set_fstart

        !save_reg_state r3
        !save_reg_state r4
        !save_reg_state r5

        %- buf: r3
        %- count: r4
        %- byte: r5

        !load_arg8 8 =buf
        !load_arg1 16 =byte
        !load_arg8 17 =count

        @loop

            cmp8 =count 0
            jmpz endloop

            mov1 [=buf] =byte

            inc =buf
            dec =count

            jmp loop

        @endloop

        !restore_reg_state r5
        !restore_reg_state r4
        !restore_reg_state r3

        ret


