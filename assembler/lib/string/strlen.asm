# strlen
# Return the length of a null-terminated string stored in r1
# The returned length excludes the null termination character
# The return value is stored in the r1 register


.text:

@@strlen

    # Store the start char* in r2
    mov r2 r1

    @ loop

        # Check if the current char is null. If so, exit the loop
        cmp1 [r1] 0
        jmpz endloop
        
        # Increment the char* and continue
        inc r1
        jmp loop


    @ endloop

    # Calculate the string length
    # r1 points to the null byte, r2 points to the start of the string
    isub

    ret

