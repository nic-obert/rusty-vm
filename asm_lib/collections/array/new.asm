.include:

    asmutils/functional.asm
    stdlib/memory.asm

    metadata.asm


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

        !set_fstart

        !save_reg_state r2
        !save_reg_state r7
        !save_reg_state r8

        %- item_size: r7
        %- length: r8

        # Load the arguments

        !load_arg8 8 =length
        !load_arg8 16 =item_size

        # Calculate the total array size
        
        mov r1 =length
        mov r2 =item_size
        imul
        
        mov1 r2 =ARRAY_METADATA_SIZE
        iadd

        # Allocate the new array
        !malloc r1


        !restore_reg_state r8
        !restore_reg_state r7
        !restore_reg_state r2

        ret

