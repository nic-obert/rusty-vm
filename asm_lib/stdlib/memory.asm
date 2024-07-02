# memory
# Library for memory management

# THIS IS DEPRECATED


.include:

    "archlib.asm"
    "asmutils/functional.asm"


.text:

    # Call malloc with the given size
    %% malloc size:

        mov8 r1 {size}
        intr =MALLOC

    %endmacro


    # Call free with the given arguemnt
    %% free addr:

        mov8 r1 {addr}
        intr =FREE

    %endmacro


    # Allocate a memory region of `num` * `size` bytes and return its address
    #
    # Args:
    #   - num: the number of elements (8 bytes)
    #   - size: the size of each element (8 bytes)
    #
    # Return:
    #   - r1: the address of the allocated memory region
    %% calloc num size:

        push8 {size}
        mov8 r1 {num}
        
        call calloc

        popsp 8

    %endmacro

    # Allocates memory block to fit num*size bytes
    # r1: number of elements
    # r2: size of element
    @@ calloc

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3

        !load_arg8 8 r2

        # Calculate the size to allocate
        # r1 is `num`, r2 is `size`
        imul

        # Save into r2 the number of bytes to initialize
        mov r2 r1

        intr =MALLOC
        
        # r1 is now the address of the memory block
        # Save the allocated block address
        mov r3 r1

        # Initialize the memory block to zeroes
        call init_zeros

        # Move the return value to r1
        mov r1 r3

        !restore_reg_state r3
        !restore_reg_state r2

        ret


    # Set from addr in r1 until size in r2 with zeros
    @ init_zeros

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

