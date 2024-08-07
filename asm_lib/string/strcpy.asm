# strcpy


.include:

    "asmutils/functional.asm"


.text:

    # Copy the null-terminated string pointed by `src into `dest`, including the termination
    # character, and stopping at that point.
    # The two memory regions should not overlap.
    #
    # Args:
    #   - src: the source string address (8 bytes)
    #   - dest: the destination memory address (8 bytes)
    #
    %% strcpy src dest:

        push8 {src}
        push8 {dest}

        call strcpy

        popsp 16

    %endmacro

    @@ strcpy

        !set_fstart

        !save_reg_state r3
        !save_reg_state r4

        %- src: r3
        %- dest: r4

        !load_arg8 8 =dest
        !load_arg8 16 =src

        @ loop

            # Copy the current char over
            mov1 [=dest] [=src]

            # Check if the char is null
            cmp1 [=src] 0
            jmpz endloop

            # The char is not null, continue
            inc =src
            inc =dest

            jmp loop

            
        @ endloop

        !restore_reg_state r4
        !restore_reg_state r3

        ret

