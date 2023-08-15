# strcpy
# Copy the null-terminated string at r1 into the buffer pointed by r2, 
# including the null termination character, and stopping at that point.
# The two buffers should not overlap.


.text:

@@ strcpy

    # Move the string pointers
    mov r3 r1
    mov r4 r2

    # Initialize the char index register
    mov1 r2 0

    @ loop

        # Get the first source char 
        mov r1 r3
        add

        mov1 r5 [r1]

        # Calculate the destination address
        mov r1 r4
        add

        # Copy the char to the destination address
        mov1 [r1] r5

        # Check if the char is null
        cmp1 r5 0
        jmpz endloop

        # The char is not null, continue
        inc r2
        jmp loop

        
    @ endloop

        ret

