
.include:

    strcmp.asm

.data:

    S1 string "hello\0"
    S2 string "helloo\0"

    equal string "Strings are equal\n\0"
    not_equal string "Strings are not equal\n\0"

.text:

@start

    # Make space for the 1-byte return value
    inc sp

    # Load the procedure arguments
    mov8 r1 S1
    mov8 r2 S2

    # Call the procedure
    call strcmp

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

