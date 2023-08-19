# strncmp
# Compare two byte strings stored in r1 and r2 up to the length in r3.
# Null bytes are treaded as normal bytes.
# Return 0 if strings are not equal
# Return 1 if strings are equal
# Return value is stored in r1


.text:

@@ strncmp

    # Initialize the char index register r8
    mov1 r8 0

    # Move the strings
    mov r4 r1
    mov r5 r2

    # Initialize the return value to 0 (strings are not equal)
    mov1 r7 0

    @ loop

        # Check if the check is finished
        cmp r8 r3
        jmpgr endloop

        # Calculate char address of s1
        mov r2 r8
        iadd

        # Store the char from s1
        mov1 r6 [r1]

        # Calculate char address of s2
        mov r1 r5
        iadd

        # Compare the chars
        cmp1 [r1] r6
        jmpnz not_equal

        # Continue the loop
        inc r8
        jmp loop


    @ not_equal
        mov1 r7 1


    @ endloop

    # Set the return value
    mov r1 r7

    ret
    
