# array
# Fixed size array allocated on the stack

# Memory structure:
#   - item_size: 8 bytes
#   - length: 8 bytes
#   - data: (item_size * length) bytes


.include:

    "asmutils/functional.asm"


.text:

    %%- ARRAY_ITEM_SIZE_OFFSET: 0
    %%- ARRAY_LENGTH_OFFSET: 8
    %%- ARRAY_DATA_OFFSET: 16


    %% array_get_item_size array: 

        mov8 r1 [{array}]

    %endmacro


    %% array_get_data_size array: 

        mov8 r1 {array}
        mov r2 =ARRAY_LENGTH_OFFSET
        iadd

        mov8 r1 [r1]

    %endmacro


    %% array_get_total_size array:

        !array_get_data_size {array}
        mov8 r2 =ARRAY_DATA_OFFSET
        iadd

    %endmacro


    # Allocate a new array on the stack
    #
    # Args:
    #   - item_size: 8 bytes
    #   - length: 8 bytes
    #
    # Return:
    #   - r1: array address (8 bytes)
    #
    # Invalidates:
    #   - r2
    #   - r3
    #
    %% array_new item_size length:

        mov r3 stp

        # Push arguments
        push8 {item_size}
        push8 {length}

        mov8 r1 {item_size}
        mov8 r2 {length}
        imul

        pushsp r1

        mov r1 r3

    %endmacro


    # Get the pointer of the array element at the given index
    #
    # Args:
    #   - r1: array pointer (8 bytes)
    #   - r2: element index (8 bytes)
    #
    # Return:
    #   - r1: element pointer (8 bytes)
    #
    @@ array_get_ptr

        !save_reg_state r3

        # Calculate the data offset
        mov r3 r1
        !array_get_item_size r3
        imul 

        # Calculate the total offset from the array pointer
        mov8 r2 =ARRAY_DATA_OFFSET
        iadd

        # Calculate the item address
        mov r2 r3
        iadd

        !restore_reg_state r3

        ret


