.include:

    asmutils/load_arg.asm
    stdlib/memory.asm


.text:

    # Allocate a new array on the heap
    #
    # Args:
    #   - item_size: 8 bytes
    #   - length: 8 bytes
    #
    # Return:
    #   - r1: array address (8 bytes)
    #
    %% array_new item_size length:

        # Push arguments
        push8 {item_size}
        push8 {length}

        call array_new

        # Pop arguments
        popsp1 16

    %endmacro
   
    @@ array_new

        # Save the current state of the register the procedure will invalidate
        push8 r2
        push8 r7
        push8 r8

        %- item_size: r7
        %- length: r8
        %- META_SIZE: 16

        # Load the arguments

        !load_arg8 8 =length
        !load_arg8 16 =item_size

        # Calculate the total array size
        
        mov r1 =length
        mov r2 =item_size
        imul
        
        mov1 r2 =META_SIZE
        iadd

        !malloc r1

        # Reload the saved register states in reverse order
        pop8 r8
        pop8 r7
        pop8 r2

        ret

