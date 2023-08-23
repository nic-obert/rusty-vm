# random password generator
# Generates a random password of the given length


.include:

    stdio.asm
    stdlib.asm
    errors.asm


.data:

    PASS_OUTPUT string "Your random password is: \0"

    INPUT_PROMPT string "Enter a password length: \0"

    INVALID_INPUT string "The given length is invalid.\0"


.text:

@start

    !print_str INPUT_PROMPT

    !input_unsigned

    mov r1 input

    # Validate the user input
    cmp1 r1 0
    jmpnz input_not_zero

        !println_str INVALID_INPUT
        !_exit =INVALID_INPUT

    @input_not_zero

    # Save the length
    mov r8 r1

    # Increment the length to store the null termination character
    inc r1

    # Allocate a buffer for the string
    !malloc r1

    # Store the string* into r6
    mov r6 r1

    # Initialize a char counter
    mov1 r7 0

    # Generate the random password
    @loop

        %- PRINTABLE_FIRST: 32
        %- PRINTABLE_LAST: 126
        %- PRINTABLE_COUNT: 95

        # Generate a random number in r1
        !rand

        # Clamp the random number in r1 into range 0..94
        mov1 r2 =PRINTABLE_COUNT
        imod

        # Move the range into 32..126
        mov1 r2 =PRINTABLE_FIRST
        iadd

        # Save the character
        mov r5 r1

        # Calculare the char address
        mov r1 r6
        mov r2 r7
        iadd

        # Copy the char into the string
        mov1 [r1] r5

        # Increment the char counter
        inc r7

        # Check if the string is finished
        cmp r7 r8
        jmpnz loop

    
    # Add the null termination character to the end of the string
    mov r1 r6
    mov r2 r8
    iadd

    mov1 [r1] '\0'

    # Print the string to the console

    !print_str PASS_OUTPUT
    !println_str r6
    
    exit

