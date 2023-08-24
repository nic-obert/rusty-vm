# strcpy


.include:

    asmutils/load_arg.asm


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

        push8 r1
        push8 r2
        push8 r3
        push8 r4
        push8 r5

        %- src: r3
        %- dest: r4
        %- i: r2

        !load_arg8 8 =dest
        !load_arg8 16 =src

        # Initialize the char index register
        mov1 =i 0

        @ loop

            # Get the current source char 
            mov r1 =src
            iadd

            # Save the char
            mov1 r5 [r1]

            # Calculate the destination address
            mov r1 =dest
            iadd

            # Copy the char to the destination address
            mov1 [r1] r5

            # Check if the char is null
            cmp1 r5 0
            jmpz endloop

            # The char is not null, continue
            inc =i
            jmp loop

            
        @ endloop


        pop8 r5
        pop8 r4
        pop8 r3
        pop8 r2
        pop8 r1

        ret

