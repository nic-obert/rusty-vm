# strcmp


.include:

    "asmutils/functional.asm"


.text:

    # Compare two null-terminated strings and return whether they are equal
    #
    # Args:
    #   - s1: the first string address (8 bytes)
    #   - s2: the second string address (8 bytes)
    #
    # Return:
    #   - r1: 1 if the strings are equal, 0 otherwise
    #
    %% strcmp s1 s2:

        push8 {s1}
        # Take advantage of r1 invalidation to use it to pass the argument without pushing it onto the stack
        mov8 r1 {s2}

        call strcmp

        popsp 8

    %endmacro

    @@ strcmp

        !set_fstart

        !save_reg_state r3
        !save_reg_state r4


        %- s1: r3
        %- s2: r4
        %- eq: r1

        !load_arg8 8 =s1
        mov =s2 r1

        # Initialize the return value to 0 (strings are not equal)
        mov1 =eq 0

        @ loop

            # Compare the chars
            cmp1 [=s1] [=s2]
            jmpnz not_equal

            # The chars are equal here
            # Check if they are null and finish
            cmp1 [=s1] 0
            jmpz equal

            # If the chars are equal but not null, increment the pointers and continue
            inc =s1
            inc =s2

            jmp loop


    @ equal
        # Set return value to 1
        mov1 =eq 1


    @ not_equal


    !restore_reg_state r4
    !restore_reg_state r3

    ret

