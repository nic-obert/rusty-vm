# strncmp


.include:

    asmutils/load_arg.asm


.text:

    # Compare two null-terminated strings up to the specified length
    #
    # Args:
    #   - s1: the first string address (8 bytes)
    #   - s2: the second string address (8 bytes)
    #   - num: the number of bytes to compare (8 bytes)
    #
    # Return:
    #   - r1: 1 if the strings are equal, 0 otherwise
    #
    %% strncmp s1 s2 num:

        push8 {s1}
        push8 {s2}
        # Use r1 since it will be invalidated
        mov8 r1 {num}

        call strncmp

        popsp1 16

    %endmacro

    @@ strncmp

        %- s1: r4
        %- s2: r5
        %- num: r3
        %- eq: r1

        mov =num r1
        !load_arg8 8 =s2
        !load_arg8 16 =s1

        # Initialize the return value to 1 (strings are equal)
        mov1 =eq 1

        @ loop

            # Check if the comparison is finished
            cmp1 =num 0
            jmpz endloop

            # Compare the chars
            cmp1 [=s1] [=s2]
            jmpnz not_equal

            # Continue the loop
            inc =s1
            inc =s2
            dec =num

            jmp loop


        @ not_equal

            mov1 =eq 0


        @ endloop

        ret
        
