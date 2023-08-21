# memswap
# Swap the two regions of memory
# r1: address of first region of memory
# r2: address of second regoin of memory
# r3: memory region size in bytes


.include:

    string/memcpy.asm


.text:

    @@ memswap

        # Save the parameters
        mov r4 r1
        mov r5 r2
        mov r6 r3

        # Allocate an intermediate buffer on the stack
        pushsp r3

        # Copy the first memory region into the buffer

        mov r2 sbp

        call memcpy

        # Copy the second memory region into the first one

        mov r1 r5
        mov r2 r4
        mov r3 r6

        call memcpy

        # Copy the temporary buffer into the second memory region

        mov r1 sbp
        mov r2 r5
        mov r3 r6

        call memcpy

        # Pop the intermediate buffer from the stack
        popsp r6

        ret

