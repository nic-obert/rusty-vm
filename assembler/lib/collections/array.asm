# array
# Fixed size array allocated on the heap

# Memory structure:
#   - item_size: 8 bytes
#   - length: 8 bytes
#   - data: (item_size * length) bytes


.include:

    stdlib/memory.asm
    asmutils/load_arg.asm
    string/memcpy.asm


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
        popsp 16

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
    #   - array: array address (8 bytes)
    #
    # Return:
    #   - r1: array length (8 bytes)
    #
    %% array_length array:

        # Save the register states
        push8 r2

        # Calculate the address of the length field
        mov8 r1 {array}
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
    #   - array: array address (8 bytes)
    #
    # Return:
    #   - r1: item size (8 bytes)
    #
    %% array_item_size array:

        # Get the item_size field
        mov8 r1 [{array}]

    %endmacro


    # Get the item in the array and push it on the stack
    #
    # Args:
    #   - array: array address (8 bytes)
    #   - index: item index to get (8 bytes)
    #
    # Return:
    #   - item on the stack: (item_size bytes)
    #
    %% array_get array index:

        # Allocate space on the stack to store the returned item
        !array_item_size {array}
        pushsp r1

        # Push arguments
        push8 {array}
        push8 {index}

        call array_get

        # Pop arguments
        popsp 16

    %endmacro

    @@ array_get

        # Save current register states
        push8 r1
        push8 r2
        push8 r5
        push8 r6
        push8 r7
        push8 r8


        %- item_addr: r5
        %- item_size: r6
        %- array: r7
        %- index: r8

        !load_arg8 8 =index
        !load_arg8 16 =array

        !array_item_size =array
        mov =item_size r1

        # Calculate the item offset (index * item_size)
        mov r2 =index
        imul

        # Caluclate the item address (array* + offset)
        mov r2 =array
        iadd
        mov =item_addr r1

        # Calculate the return value address
        mov r1 sbp
        mov1 r2 24
        isub

        # Copy the return value onto the stack
        !memcpy =item_addr r1 =item_size


        # Restore previous register states
        pop8 r8
        pop8 r7
        pop8 r6
        pop8 r5
        pop8 r2
        pop8 r1

        ret

