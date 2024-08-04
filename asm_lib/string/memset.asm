
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

    %endmacro


    @@ memset:

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3

        %- dest: r1
        %- count: r2
        %- byte: r3

        !load_arg8 8 =dest
        !load_arg1 9 =byte
        !load_arg1 17 =size

        @loop

            cmp8 =num 0
            jmpz endloop
            
            mov1 [=dest] =byte

            inc =dest

            jmp loop

        @endloop

        !restore_reg_state r3
        !restore_reg_state r2
        !restore_reg_state r1

        ret

