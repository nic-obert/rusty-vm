# rand
# Random number generation


.include:

    "archlib.asm"
    "asmutils/functional.asm"

.text:

    # Generate a random 8-byte number
    #
    # Return:
    #   - r1: the random number
    #
    %% rand:

        mov1 int =RANDOM
        intr
    
    %endmacro


    # Generate a random integer number between `start` and `end`
    #
    # Args:
    #   - start: the start of the range
    #   - end: the end of the range
    #
    # Return:
    #   - r1: the random number
    #
    %% rand_range start end:

        push8 {start}
        push8 {end}

        call rand_range

        popsp1 16

    %endmacro

    @@ rand_range

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- start: r3
        %- end: r4

        !load_arg8 8 =end
        !load_arg8 16 =start

        # Calculate the range size and save it in r2 for later use
        mov r1 =end
        mov r2 =start
        isub
        mov r2 r1

        # Generate the random number
        !rand

        # Clamp the random number in range
        imod

        # Shift the range back in place
        mov r2 =start
        iadd

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret

