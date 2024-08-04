
.text:

    # Set the buffer to be all zeroes
    #
    # Args:
    #   - r1: buffer start pointer (8 bytes)
    #   - r2: buffer size in bytes (8 bytes)
    #
    @@ set_zeros

        @loop

            # Check if finished
            cmp1 r2 0
            jmpz endloop

            # Set byte to 0
            mov1 [r1] 0

            inc r1
            dec r2

            jmp loop

        @endloop

        ret

