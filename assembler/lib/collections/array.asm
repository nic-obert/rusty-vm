# array
# Fixed size array allocated on the heap

# Memory structure:
#   - item_size: 8 bytes
#   - length: 8 bytes
#   - data: (item_size * length) bytes



.include:

    stdlib/memory.asm
    asmutils/load_arg.asm


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

        push8 {item_size}
        push8 {length}

        call array_new

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


    # Return the length of the array
    #
    # Args:
    #   - arr: array address (8 bytes)
    #
    # Return:
    #   - r1: array length (8 bytes)
    #
    %% array_length arr:

        # Save the register states
        push8 r2

        # Calculate the address of the length field
        mov8 r1 {arr}
        mov1 r2 8
        iadd

        # Get the length field
        mov8 r1 [r1]

        # Restore the register states
        pop8 r2

    %endmacro


    # Return the item size of the array
    #
    # Args:
    #   - arr: array address (8 bytes)
    #
    # Return:
    #   - r1: item size (8 bytes)
    #
    %% array_item_size arr:

        # Get the item_size field
        mov8 r1 [r1]

    %endmacro

