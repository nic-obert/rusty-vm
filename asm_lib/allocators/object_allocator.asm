# Fixed-size object allocator
#
# Heap structure:
# 0x0000 ------------------------------------------------------------------------------ 0xFFFF
# [ Heap end ptr (8 bytes) ] [ Object size (8 bytes) ] [ ------ Cells ------> ] [ <--- Stack ]
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

    @heap_overflow_message
    dcs "Heap overflow"

    @double_free_message
    dcs "Double free"

    @free_out_of_heap_message
    dcs "Free out of heap"

    @misaligned_free_message
    dcs "Misaligned free"


.text:

    %- META_HEAP_END_PTR: 0
    %- META_OBJ_SIZE_PTR: 8
    %- HEAP_START: 16
    %- CELL_FREE: 0
    %- CELL_OCCUPIED: 1


    # r1: non-zero object size
    # r2: non-zero max object count
    #
    @@ init_object_allocator

        # Check r1 != 0 && r2 != 0
        cmp8 r1 0
        jmpnz r1_ok
            mov1 error =INVALID_INPUT
            ret
        @r1_ok
        cmp8 r2 0
        jmpnz r2_ok
            mov1 error =INVALID_INPUT
            ret
        @r2_ok

        !save_reg_sate r1
        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4

        %- OBJ_SIZE: r3
        %- OBJ_COUNT: r4

        mov =OBJ_SIZE r1
        mov =OBJ_COUNT r2

        mov8 r1 =HEAP_START

        @initializing

            mov1 [r1] =CELL_FREE
            inc r1
            mov8 r2 =OBJ_SIZE
            iadd

            dec =OBJ_COUNT
            jmpnz initializing

        # Write the heap end ptr to heap metadata
        # (obj_size + cell_meta_size) * cell_count + heap_start
        mov r1 =OBJ_SIZE
        inc r1
        mov r2 =OBJ_COUNT
        imul
        mov8 r2 =HEAP_START
        iadd
        mov8 [=META_HEAP_END_PTR] r1

        # Write object size to heap metadata
        mov8 [=META_OBJ_SIZE_PTR] =OBJ_SIZE

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

        %- CURSOR: r1

        mov8 =CURSOR =HEAP_START

        @searching

            # Check for heap overflow
            cmp8 =CURSOR [=HEAP_META_END_PTR]
            jmpge panic_heap_overflow

            # If cell is occupied, continue search
            cmp1 [=CURSOR] =CELL_OCCUPIED
            jmpnz searching

        # Found free cell
        # Mark as occupied
        mov1 [=CURSOR] =CELL_OCCUPIED
        # Increment the pointer to access the cell's data
        inc =CURSOR

        # Remember that r1 is =CURSOR, so the result is already in r1

        ret


    # r1: object address
    #
    # Panics if the provided address is not in the heap.
    # Panics in case of double free.
    # Panics if the provided address is misaligned.
    #
    @@ free_object

        # Check r1 > heap_start && r1 < heap_end && r1 % (obj_size + cell_meta_size) == 0
        # r1 > heap_start
        cmp8 r1 [=META_HEAP_START]
        jmpgr



        ret


    @ panic_heap_overflow

        !println_str heap_overflow_message
        mov1 error =OUT_OF_MEMORY
        exit
