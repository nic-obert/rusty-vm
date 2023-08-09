
.include:

    strcmp.asm

.data:

    s1 string "Hello\0"
    s2 string "World\0"

    equal string "Strings are equal\n\0"
    not_equal string "Strings are not equal\n\0"

.text:

@start

    # Make space for the 1-byte return value
    inc sp

    # Push the arguments to the stack
    push8 s1
    push8 s2

    # Call the function
    jmp strcmp

    # Get the return value
    pop1 r1

    jmpz ifne r1

    mov8 print equal
    jmp endif
    @ifne
        mov8 print not_equal
    @endif

    printstr

    exit

