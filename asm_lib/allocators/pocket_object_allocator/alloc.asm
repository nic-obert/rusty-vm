.include:

    "shared.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "stdio/print.asm"

.data:

    @out_of_memory_message
    dcs "No free cell for allocation"


.text:

# Args:
#   - r1: allocator handle
#
# Return:
#   - r1: non-null allocated block address
#
# Panics if out of memory
#
@@ pocket_alloc_object

    # Linear search for free cells

    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4

    %- OBJ_SIZE: r3
    %- HEAP_END_PTR: r4

    # Read the heap metadata
    mov8 =OBJ_SIZE [r1]
    mov8 r2 =OBJ_SIZE_FIELD_SIZE
    iadd
    mov8 =HEAP_END_PTR [r1]
    mov8 r2 =ADDRESS_SIZE
    iadd

    @searching

        # Check if we ran out of cells
        cmp r1 =HEAP_END_PTR
        jmpge panic_out_of_memory

        # If cell is free, end search
        cmp1 [r1] =CELL_FREE
        jmpz end_search

        # Calculate the new cursor
        mov r2 =OBJ_SIZE
        iadd
        inc r1
        jmp searching

    @end_search
    # Found free cell
    # Mark as occupied
    mov1 [r1] =CELL_OCCUPIED
    # Increment the pointer to access the cell's data (r1 is returned)
    inc r1

    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2

    ret


@ panic_out_of_memory
    !println_str out_of_memory_message
    mov1 error =OUT_OF_MEMORY
    exit
