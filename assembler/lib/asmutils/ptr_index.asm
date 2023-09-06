# ptr_index
# Macros for indexing pointers


.include:

    asmutils/functional.asm


.text:

    # Dereference `ptr` at `index` into r1, treating the pointer as an array of 1-byte-sized objects
    #
    # Args:
    #   - ptr: the pointer to index (8 bytes)
    #   - index: the index (8 bytes)
    #
    # Return:
    #   - r1: the dereferenced value
    #
    %% ptr_index1 ptr index:

        push8 {ptr}
        push8 {index}
        
        call ptr_index1

        popsp1 16
    
    %endmacro

    @@ ptr_index1

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- ptr: r3
        %- index: r4

        !load_arg8 8 =index
        !load_arg8 16 =ptr

        # The offset is already `index` for indexing of size 1

        mov r1 =ptr
        mov r2 =index
        iadd

        # Dereference pointer
        mov1 r1 [r1]

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret

    
    # Dereference `ptr` at `index` into r1, treating the pointer as an array of 2-byte-sized objects
    #
    # Args:
    #   - ptr: the pointer to index (8 bytes)
    #   - index: the index (8 bytes)
    #
    # Return:
    #   - r1: the dereferenced value
    #
    %% ptr_index2 ptr index:

        push8 {ptr}
        push8 {index}
        
        call ptr_index2

        popsp1 16
    
    %endmacro

    @@ ptr_index2

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- ptr: r3
        %- index: r4

        !load_arg8 8 =index
        !load_arg8 16 =ptr

        # Calculate the offset
        mov r1 =index
        mov1 r2 2
        imul

        # Calculate the data address
        mov r2 =ptr
        iadd

        # Dereference pointer
        mov2 r1 [r1]

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret


    # Dereference `ptr` at `index` into r1, treating the pointer as an array of 4-byte-sized objects
    #
    # Args:
    #   - ptr: the pointer to index (8 bytes)
    #   - index: the index (8 bytes)
    #
    # Return:
    #   - r1: the dereferenced value
    #
    %% ptr_index4 ptr index:

        push8 {ptr}
        push8 {index}
        
        call ptr_index4

        popsp1 16
    
    %endmacro

    @@ ptr_index4

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- ptr: r3
        %- index: r4

        !load_arg8 8 =index
        !load_arg8 16 =ptr

        # Calculate the offset
        mov r1 =index
        mov1 r2 4
        imul

        # Calculate the data address
        mov r2 =ptr
        iadd

        # Dereference pointer
        mov4 r1 [r1]

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret


    # Dereference `ptr` at `index` into r1, treating the pointer as an array of 8-byte-sized objects
    #
    # Args:
    #   - ptr: the pointer to index (8 bytes)
    #   - index: the index (8 bytes)
    #
    # Return:
    #   - r1: the dereferenced value
    #
    %% ptr_index8 ptr index:

        push8 {ptr}
        push8 {index}
        
        call ptr_index8

        popsp1 16
    
    %endmacro

    @@ ptr_index8

        !set_fstart

        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- ptr: r3
        %- index: r4

        !load_arg8 8 =index
        !load_arg8 16 =ptr

        # Calculate the offset
        mov r1 =index
        mov1 r2 8
        imul

        # Calculate the data address
        mov r2 =ptr
        iadd

        # Dereference pointer
        mov8 r1 [r1]

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2

        ret

