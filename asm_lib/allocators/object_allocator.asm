# Fixed-size object allocator
#
# Heap structure:
# 0x0000 -------------------------- 0xFFFF
# [ ------ Cells ------> ] [ <--- Stack ]
# Cell structure:
# [ Free (1 byte) ] [ Cell data ]
#
# Warning: don't use this allocator along with other heap allocators
#


.include:

    "archlib.asm"
    "stddef.asm"
    "asmutils/functional.asm"
    "stdio/print.asm"


.data:

    @out_of_memory_message
    dcs "No free cell for allocation"

    @double_free_message
    dcs "Double free"

    @free_out_of_heap_message
    dcs "Free out of heap"

    @misaligned_free_message
    dcs "Misaligned free"

    @zero_size_obj_message
    dcs "Attempting to initialize an object allocator with objects of size 0"

    @zero_obj_count_message
    dcs "Attempting to initialize an object allocator with a maximum of 0 objects"

    @meta_heap_end_ptr
    dn 8 0
    @meta_obj_size_ptr
    dn 8 0


.text:

    %- CELL_FREE: 0
    %- CELL_OCCUPIED: 1


    # r1: non-zero object size
    # r2: non-zero max object count
    #
    # Panics if inputs are invalid
    @@ init_object_allocator

        # Check r1 != 0 && r2 != 0
        cmp8 r1 0
        jmpz panic_zero_size
        cmp8 r2 0
        jmpz panic_zero_count

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- OBJ_SIZE: r3
        %- OBJ_TO_INITIALIZE: r4

        mov =OBJ_SIZE r1
        mov =OBJ_TO_INITIALIZE r2

        # We trust that pep remains valid for the lifetime of the program
        mov r1 pep

        @initializing

            mov1 [r1] =CELL_FREE
            inc r1
            mov8 r2 =OBJ_SIZE
            iadd

            dec =OBJ_TO_INITIALIZE
            jmpnz initializing

        # Write the heap end ptr to heap metadata
        mov8 [meta_heap_end_ptr] r1

        # Write object size to heap metadata
        mov8 [meta_obj_size_ptr] =OBJ_SIZE

        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2
        !restore_reg_state r1

        ret


    # Return:
    # - r1: non-null allocated block address
    #
    # Panics if out of memory
    #
    @@ alloc_object

        # Linear search for free cells

        !save_reg_state r2

        %- CURSOR: r1

        mov =CURSOR pep

        @searching

            # Check if we ran out of cells
            cmp8 =CURSOR [meta_heap_end_ptr]
            jmpge panic_out_of_memory

            # If cell is free, end search
            cmp1 [=CURSOR] =CELL_FREE
            jmpz end_search

            # Calculate the new cursor
            mov8 r2 [meta_obj_size_ptr]
            # Note =CURSOR is r1
            iadd
            inc =CURSOR
            jmp searching

        @end_search
        # Found free cell
        # Mark as occupied
        mov1 [=CURSOR] =CELL_OCCUPIED
        # Increment the pointer to access the cell's data
        inc =CURSOR

        # Remember that r1 is =CURSOR, so the result is already in r1

        !restore_reg_state r2

        ret


    # r1: object address
    #
    # Panics if the provided address is not in the heap.
    # Panics in case of double free.
    # Panics if the provided address is misaligned.
    #
    @@ free_object

        %- ADDR: r3
        %- TMP_NORMALIZED_ADDR: r4

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        mov =ADDR r1

        # Check addr > heap_start && addr < heap_end && (addr - heap_start) % (obj_size + cell_meta_size) == 0
        #
        # addr > heap_start
        cmp8 =ADDR pep
        jmple panic_free_out_of_heap

        # addr < heap_end
        cmp8 =ADDR [meta_heap_end_ptr]
        jmpge panic_free_out_of_heap

        # (addr - heap_start - cell_meta_size) % (obj_size + cell_meta_size) == 0
        #mov r1 =ADDR
        mov8 r2 pep
        isub
        dec r1
        mov =TMP_NORMALIZED_ADDR r1
        mov8 r1 [meta_obj_size_ptr]
        inc r1
        mov r2 r1
        mov r1 =TMP_NORMALIZED_ADDR
        imod
        jmpnz panic_misaligned_free

        # Actual free algorithm

        mov r1 =ADDR
        dec r1
        cmp1 [r1] =CELL_FREE
        jmpz panic_double_free

        mov1 [r1] =CELL_FREE


        !restore_reg_state r4
        !restore_reg_state r3
        !restore_reg_state r2
        !restore_reg_state r1

        ret


    @ panic_out_of_memory
        !println_str out_of_memory_message
        mov1 error =OUT_OF_MEMORY
        exit

    @ panic_free_out_of_heap
        !println_str free_out_of_heap_message
        mov1 error =OUT_OF_BOUNDS
        exit

    @ panic_double_free
        !println_str double_free_message
        mov1 error =INVALID_INPUT
        exit

    @ panic_misaligned_free
        !println_str misaligned_free_message
        mov1 error =INVALID_INPUT
        exit

    @ panic_zero_size
        !println_str zero_size_obj_message
        mov1 error =INVALID_INPUT
        exit

    @ panic_zero_count
        !println_str zero_obj_count_message
        mov1 error =INVALID_INPUT
        exit
