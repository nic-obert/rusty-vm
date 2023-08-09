# Compare two null-terminated strings
# Return 0 if strings are not equal
# Return 1 if strings are equal
# Return value is 1 byte large
# Function call memory structure
# | return value | return address | args |


.text:

@@strcmp

    # Initialize the character index register
    mov8 r8 0

    # Get the first string address from the stack and consume it
    pop8 r3
    
    # Get the second string address from the stack and consume it
    pop8 r4

    # Now sp points to after the return address

    # Initialize the return value to 0 (strings are not equal)
    mov r1 sp
    mov1 r2 9
    add

    # r1 now stores the address of the return value

    # Set return value to 0
    mov1 [r1] 0

    # Store the return value address
    mov r7 r1

    @loop

        # Calculate the address of the char of s1
        mov r2 r8
        mov r1 r3
        add

        # Store the char of s1 to compare
        mov1 r5 [r1]

        # Calculate the address of the char from s2
        mov r1 r3
        # r2 is still the char index
        add

        # Compare the chars
        cmp1 r5 [r1]

        # If the chars are equal, zf is 1, else 0
        
        # If the chars are different, return
        jmpz endloop zf

        # The chars are equal, check if they are null and finish
        jmpz equal r5

        # If the chars are equal but not null, continue
        inc r8
        jmp loop

    @equal
        # Set return value to 1
        mov1 [r7] 1

    @endloop

    # Get the return address and consume it from the stack
    pop8 r1

    # Return to the caller
    jmp r1

    # Everything has been popped already     

