# memory
# Library for memory management


.include:

    @@ interrupts.asm


.text:

    # Call malloc with the given size
    %% malloc size:

        mov8 r1 {size}
        intr [MALLOC]

    %endmacro


    # Call free with the given arguemnt
    %% free addr:

        mov8 r1 {addr}
        intr [FREE]

    %endmacro


    # Call calloc with the given arguments
    %% calloc num size:

        mov8 r1 {num}
        mov8 r2 {size}
        
        call calloc

    %endmacro


    # Allocates memory block to fit num*size bytes
    # r1: number of elements
    # r2: size of element
    @@ calloc

        # Calculate the size to allocate
        imul

        # Save into r2 the bytes to initialize
        mov r2 r1

        intr [MALLOC]
        
        # r1 is now the address of the memory block

        # Initialize the memory block to zeroes
        call init_zeros

        ret


    # Set from addr in r1 until size in r2 with zeros
    @@ init_zeros

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

