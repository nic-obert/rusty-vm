.include:

    "shared.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "stdio/print.asm"


.data:

    @zero_size_obj_message
    dcs "Attempting to initialize an object allocator with objects of size 0"

    @zero_obj_count_message
    dcs "Attempting to initialize an object allocator with a maximum of 0 objects"


.text:

# Args:
#   - r1: non-zero object size
#   - r2: non-zero max object count
#   - pep: address where the heap allocator will be instantiated
#
# Returns:
#   - r1: allocator handle
#   - pep: end address of the heap allocator
#
# Panics if inputs are invalid
#
@@ init_pocket_object_allocator

    # Check r1 != 0 && r2 != 0
    cmp8 r1 0
    jmpz panic_zero_size
    cmp8 r2 0
    jmpz panic_zero_count

    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4
    !save_reg_state r5
    !save_reg_state r6

    %- OBJ_SIZE: r3
    %- OBJ_TO_INITIALIZE: r4
    %- HEAP_METADATA_START: r5
    %- HEAP_END_PTR_ADDR: r6

    mov =OBJ_SIZE r1
    mov =OBJ_TO_INITIALIZE r2
    mov =HEAP_METADATA_START pep
    mov8 [=HEAP_METADATA_START] =OBJ_SIZE

    mov r1 =HEAP_METADATA_START
    mov8 r2 =OBJ_SIZE_FIELD_SIZE
    iadd
    mov =HEAP_END_PTR_ADDR r1

    mov8 r2 =ADDRESS_SIZE
    iadd

    # r1 is now the heap buffer start

    @initializing

        mov1 [r1] =CELL_FREE
        inc r1
        mov8 r2 =OBJ_SIZE
        iadd

        dec =OBJ_TO_INITIALIZE
        jmpnz initializing

    # Write the heap end ptr to heap metadata
    mov8 [=HEAP_END_PTR_ADDR] r1
    # Update the pep register to signal the heap end
    mov pep r1

    # Set return value
    mov r1 =HEAP_METADATA_START

    !restore_reg_state r6
    !restore_reg_state r5
    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2

    ret


@ panic_zero_size
    !println_str zero_size_obj_message
    mov1 error =INVALID_INPUT
    exit

@ panic_zero_count
    !println_str zero_obj_count_message
    mov1 error =INVALID_INPUT
    exit
