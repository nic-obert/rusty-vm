# str_from_int

# DEPRECATED


.include:

    "stdlib/memory.asm"
    "stdio/print.asm"
    "asmutils/functional.asm"


.text:

    # Convert the unsigned integer `num` into a heap-allocated null-terminated string
    #
    # Args:
    #   - num: the unsigned integer to convert
    #
    # Return:
    #   - r1: the returned string address (8 bytes)
    #
    %% str_from_uint num:

        mov8 r1 {num}

        call str_from_uint

    %endmacro

    @@ str_from_uint

        # TODO: eventually, implement this using log10(number)

        # No need to !set_fstart since there are no args to load

        !save_reg_state r2
        !save_reg_state r6
        !save_reg_state r7
        !save_reg_state r8

        %- num: r8
        %- len: r7
        %- str: r6

        # Save the number
        mov =num r1

        # Initialize the length counter to 1 to account for the null termination
        mov1 =len 1

        # If the number is 0, increment the length by 1 because it won't be counted as a display_registers
        cmp1 =num 0
        jmpnz not_zero

            inc =len

        @not_zero

        # The number will be divided by 10 multiple times
        mov1 r2 10


        # Calculate the length of the number
        @length_loop

            # Check if the number is finished
            cmp1 r1 0
            jmpz length_endloop

            idiv

            inc =len
            jmp length_loop

        @length_endloop


        # Allocate the memory buffer for the string and store it in r6
        !malloc =len

        # Save the string address
        mov =str r1

        # Decrement length because indices start at 0
        dec =len

        # Transform the string length into a pointer to the current character
        # r1 is still the string address
        %- cc: =len
        mov r2 =len
        iadd
        mov =cc r1

        # Set the last byte of the string to the null character
        mov1 [=cc] '\0'

        dec =cc


        # Convert the number to a string


        @convert_loop

            # Get the numbers back into the registers
            mov r1 =num
            mov1 r2 10

            imod

            # r1 is now the digit

            # Convert the digit to a char digit
            mov1 r2 48
            iadd

            # Copy the char into the string
            mov1 [=cc] r1

            dec =cc

            # Divide the number by 10 to remove the digit
            mov r1 =num
            mov1 r2 10
            idiv

            mov =num r1

            # Check if this is the last char
            cmp =cc =str
            jmpge convert_loop

        
        # Load the string address into r1 and return
        mov r1 =str


        !restore_reg_state r8
        !restore_reg_state r7
        !restore_reg_state r6
        !restore_reg_state r2

        ret

    