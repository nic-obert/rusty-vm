# ascii_to_int
# Convert an ASCII string to an integer.
# Ignore the whitespace characters before the first non-whitespace character.
# Ignore the rest of the string after the first numeric character sequence.
# If no numeric characters are found after the first non-whitespace sequence, return 0.
# Input string address stored in r1.
# Store output in r1.


.include:

    ctype.asm
    stdlib/ascii_to_digit.asm


.text:

@@ ascii_to_int

    # Move char*
    mov r3 r1

    # Initialize output
    mov1 r7 0

    # Initialize sign (default is positive 0)
    mov1 r8 0

    # Skip the whitespaces
    @ loop_whitespaces

        # Load the chat
        mov1 r1 [r3]

        call is_space

        # If it's not a whitespace, break the loop
        cmp1 r1 0
        jmpz endloop_whitespaces

        inc r3
        jmp loop_whitespaces

    @ endloop_whitespaces


    # Check if the current char is a -
    cmp1 [r3] '-'
    jmpnz char_not_minus

        # Save the negative sign (1 is negative)
        mov1 r8 1
        inc r3

    @ char_not_minus


    # Check if the current char is a +
    cmp1 [r3] '+'
    jmpnz char_not_plus

        inc r3

    @ char_not_plus


    # Construct the number from the string

    @ loop_num

        # Load the char in r1
        mov1 r1 [r3]

        # Try to convert the char to digit
        call ascii_to_digit

        # Check for errors
        cmp1 error INVALID_INPUT

        # Clean error register
        mov1 error NO_ERROR

        jmpz endloop

        # If the char is a valid digit, add it to the number

        # Save the digit in r4
        mov r4 r1

        # output * 10
        mov r1 r7
        mov1 r2 10
        mul
        
        # output + digit
        mov r2 r4
        mul

        # Save current output
        mov r7 r1


    @ endloop


    # Prepare return value 
    mov r1 r7

    ret

