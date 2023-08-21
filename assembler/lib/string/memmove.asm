# memmove
# r1: source address
# r2: destination address
# r3: bytes to copy
# Allow safe copy if source and destination memory regions overlap


.include:

    string/memcpy.asm


.text:

@@ memmove

    # Move pointers
    mov r4 r1
    mov r5 r2

    # Check if source and destination regions unsafely overlap
    # Unsafe overlap configuration is (src < dest && src + n > dest)
    # Other overlapping configurations are safe for memcpy

    # Filter out non-overlapping and safely overlapping configurations

    # src < dest
    cmp r1 r2
    jmpge no_overlap

    # src + n
    mov r2 r3
    iadd

    # src + n > dest
    cmp r1 r5
    jmple no_overlap


# Unsafe overlap case

    # Allocate the intermediate buffer of n bytes on the stack
    pushsp r3

    # Load the addresses
    mov r1 r4
    mov r2 sbp

    # Save number of bytes to copy
    mov r8 r3

    # Copy source into intermediate buffer
    call memcpy

    # Change source from original address to intermediate buffer
    mov r1 sbp
    # Put number of bytes to copy back into r3
    mov r3 r8

    # Reload the destination address
    mov r2 r5

    # Copy intermediate buffer into destination
    call memcpy 

    # Pop the intermediate buffer after it's being used
    popsp r8

    ret


@ no_overlap

    # Load the pointers 
    mov r1 r4
    mov r2 r5

    # Copy source (or buffer) into destination
    call memcpy

    ret

