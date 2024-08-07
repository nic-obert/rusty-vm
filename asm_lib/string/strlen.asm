# strlen

.include:

    "asmutils/functional.asm"


.text:

    # Return the length of a null-terminated string
    #
    # Args:
    #   - str: string address (8 bytes)
    #
    # Return:
    #   - r1: length of the string in bytes
    #
    %% strlen str:

        mov8 r1 {str}

        call strlen

    %endmacro

    @@ strlen
        
        !save_reg_state r2

        # Store the start char* in r2
        mov r2 r1

        @ loop

            # Check if the current char is null. If so, exit the loop
            cmp1 [r1] 0
            jmpz endloop
            
            # Increment the current char* and continue
            inc r1
            jmp loop


        @ endloop

        # Calculate the string length
        # r1 points to the null byte, r2 points to the start of the string
        isub

        !restore_reg_state r2

        ret

