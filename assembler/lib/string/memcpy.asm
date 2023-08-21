# memcpy
# Copy r3 bytes from the memory location r1 into the memory location r2
# Source and destination memory buffers should not overlap
# r1: source address
# r2: destination address
# r3: bytes to copy


.text:

@@ memcpy

    @ loop

        # Check if all bytes have been copied
        cmp1 r3 0
        jmpz endloop

        # Copy the byte
        mov1 [r2] [r1]

        # Increment the byte pointers
        inc r1
        inc r2

        # Decrement the bytes still to write
        dec r3

        jmp loop

    
    @ endloop

        ret

