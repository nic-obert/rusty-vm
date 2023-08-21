# str_from_int
# Converts the unsigned integer in r1 into a null-terminated string
# The returned string is stored on the heap
# r1: num


.include:

    stdlib/memory.asm
    stdio/print.asm


.text:

    @@ str_from_uint

        # TODO: eventually, implement this using log10(number)

        # Save the number
        mov r8 r1

        # Initialize the length counter (1 to account for the null termination)
        mov1 r7 1

        # If the number is 0, increment the length by 1 because it won't be counted as a display_registers
        cmp1 r1 0
        jmpnz not_zero

            inc r7

        @not_zero

        # The number will be divided by 10 multiple times
        mov1 r2 10


        # Calculate the length of the number
        @length_loop

            # Check if the number is finished
            cmp1 r1 0
            jmpz length_endloop

            idiv

            inc r7
            jmp length_loop

        @length_endloop


        # Allocate the memory buffer for the string and store it in r6
        !malloc r7

        # Decrement length because indices start at 0
        dec r7

        mov r6 r1

        # Transform the string length into a pointer to the current character
        mov r2 r7
        iadd

        mov r7 r1

        # Set the last byte of the string to the null character
        mov1 [r7] '\0'

        dec r7


        # Convert the number to a string


        @convert_loop

            # Get the numbers back into the registers
            mov r1 r8
            mov1 r2 10

            imod

            # r1 is now the digit

            # Convert the digit to a char digit
            mov1 r2 48
            iadd

            # Copy the char into the string
            mov1 [r7] r1

            dec r7

            # Divide the number by 10 to remove the digit
            mov r1 r8
            mov1 r2 10
            idiv

            mov r8 r1

            # Check if this is the last char
            cmp r7 r6
            jmpge convert_loop

        
        # Load the string address into r1 and return
        mov r1 r6

        ret

    