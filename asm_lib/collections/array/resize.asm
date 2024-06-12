.include:

    asmutils/functional.asm
    string/memcpy.asm
    stdlib/memory.asm

    item_size.asm
    length.asm
    shared.asm
    metadata.asm


.text:

    # Resize the array to be of length `new_size`.
    # Creates a new array of the given size and copies the old array's elements over.
    # Deallocates the old array.
    # The new added array slots are uninitialized. 
    # Downsizing an array is undefined behavior.
    #
    # Args:
    #   - array: array address (8 bytes)
    #   - new_len: the new array length (8 bytes)
    #
    # Return:
    #   - r1: address of the new array
    #
    %% array_resize array new_len:

        push8 {array}
        push8 {new_len}

        call array_resize

        popsp1 16

    %endmacro

    @@ array_resize

        !set_fstart

        !save_reg_state r2
        !save_reg_state r4
        !save_reg_state r5
        !save_reg_state r6
        !save_reg_state r7
        !save_reg_state r8

        %- old_array: r8
        %- new_array: r7
        %- new_len: r6
        %- old_len: r5
        %- old_size: r4

        !load_arg8 8 =new_len
        !load_arg8 16 =old_array

        !array_length =old_array
        mov =old_len r1

        # Calculate the old array data size
        !array_item_size array
        mov r2 =old_len
        imul
        mov =old_size r1

        # Calculate the new array data size
        !array_item_size =array
        mov r2 =new_len
        imul

        # Add the metadata size to get the total new size
        mov1 r2 =ARRAY_METADATA_SIZE
        iadd

        # Allocate the new array
        # r1 is the new size
        !malloc r1
        mov =new_array r1

        # Write the metadata into the new array
        mov8 [=new_array] =item_size
        
        !array_get_length_ptr =new_array
        mov8 [r1] =new_len

        # Copy the data from the old array over to the new

        !array_get_data_ptr =new_array
        mov r2 r1

        !array_get_data_ptr =old_array

        !memcpy r1 r2 =old_size

        # Deallocate the old array
        !free =old_array

        # Return the new array
        mov r1 =new_array


        !restore_reg_state r8
        !restore_reg_state r7
        !restore_reg_state r6
        !restore_reg_state r5
        !restore_reg_state r4
        !restore_reg_state r2
                
        ret

