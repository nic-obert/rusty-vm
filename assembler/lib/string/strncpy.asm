# strncpy
# Copy the number of characters specified in r3 from string in r1 over to string in r2
# If the null termination is reached before writing the requested amount of characters,
# the destination string is padded with zeroes until the requested amount of characters are is written.
# The resulting string shall not be considered null-terminated, as the null termination may not be copied.


.text:

@@ strncpy

    @ loop_copy

        # Check if the requested chars have been copied
        cmp1 r3 0
        jmpz endloop

        # Copy the chars
        mov1 [r2] [r1]

        # Check if the source char is null (source string is finished)
        cmp1 [r1] 0
        jmpz pad_with_zeroes

        # Increment the char*
        inc r1
        inc r2

        # Decrement the chars still to write
        dec r3

        jmp loop_copy


@ pad_with_zeroes

    @ loop_pad

        # Check if the requested chars have been copied
        cmp1 r3 0
        jmpz endloop

        # Write 0 to destination
        mov1 [r2] 0

        # Increment the char*
        inc r2

        # Decrement the chars still to write
        dec r3

        jmp loop_pad


@ endloop

    ret