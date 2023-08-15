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

    # Check if source and destination regions overlap

    # src + n
    mov r2 r3
    add

    # src + n > dest
    cmp r1 r5
    jmpgr overlap

    # dest + n
    mov r1 r5
    add

    # dest + n > src
    cmp r1 r4
    jmpgr overlap

    jmp no_overlap


@ overlap

    # Copy to an intermediate buffer on the stack

    mov r1 r4
    mov r2 sp

    # Save number of bytes to copy
    mov r8 r3

    call memcpy

    # Change source from original to buffer
    mov r4 sp
    # Put number of bytes to copy back into r3
    mov r3 r8


@ no_overlap

    # Load the pointers 
    mov r1 r4
    mov r2 r5

    # Copy source (or buffer) into destination
    call memcpy

    ret

