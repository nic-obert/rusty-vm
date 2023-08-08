
.data:

    s1 string "nic\0"
    s2 string "noc\0"

    result_equal string "The strings are equal\0"
    result_not_equal string "The strings are not equal\0"

.text:

@start

    # Compare null-terminated strings

    # Initialize the character index register
    mov8 r3 0

    # Initialize the result register (return 1 if strings are equal, return 0 in strings are not equal)
    mov8 r8 0

    # Create a loop to check compare the strings

    @loop

        # Check if the current characters are null
        
        # Calculate the address of the character of s1
        mov8 r1 s1
        mov r2 r3
        add

        # Store the character of s1 to compare
        mov1 r4 [r1]
        
        # Calculate the address of the character of s2
        mov8 r1 s2
        mov r2 r3
        add

        # Compare the two characters from s1 and s2
        mov r2 r4
        cmp r1 r2

        # If the characters are equal, zf is 1, otherwize zf is 0
        # If the characters are different, return
        jmpz endloop zf

        # If the characters are equal, check if the strings reached the end
        # Since the characters are equal, we can check only one for null termination
        jmpz equal r1

        # If the characters are equal but the string isn't finished, continue the loop
        inc r3
        jmp loop

    @equal
        mov1 r8 1

    @endloop

    # Output the comparison result

    jmpz if_not_equal r8

    #@if_equal
        mov8 print result_equal
        jmp endif
    @if_not_equal
        mov8 print result_not_equal
    @endif

    printstr


