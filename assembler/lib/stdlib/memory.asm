# memory
# Library for memory management


.include:

    @@ interrupts.asm


.text:

    %% malloc size:

        mov8 r1 {size}
        intr [MALLOC]

    %endmacro


    %% free addr:

        mov8 r1 {addr}
        intr [FREE]

    %endmacro


    %% calloc num size:

        # Calculate the size to allocate
        mov8 r1 {num}
        mov8 r2 {size}
        imul

        intr [MALLOC]

        # Initialize the memory block to zeroes
        #TODO

    %endmacro

