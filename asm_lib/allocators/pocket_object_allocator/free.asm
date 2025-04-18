.include:

    "archlib.asm"
    "shared.asm"
    "asmutils/functional.asm"
    "stdio/print.asm"

.data:

    @double_free_message
    dcs "Double free: "

    @free_out_of_heap_message
    dcs "Free out of heap: "

    @misaligned_free_message
    dcs "Misaligned free: "


.text:

# Args:
#   - r1: allocator handle
#   - r2: object address
#
# Panics if the provided address is not in the heap.
# Panics in case of double free.
# Panics if the provided address is misaligned.
#
@@ free_object

    !save_reg_state r1
    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4
    !save_reg_state r5
    !save_reg_state r6
    !save_reg_state r7

    %- ADDR: r3
    %- TMP_NORMALIZED_ADDR: r4
    %- OBJ_SIZE: r5
    %- HEAP_END_PTR: r6
    %- HEAP_START: r7

    mov =ADDR r2

    # Load heap metadata
    mov8 =OBJ_SIZE [r1]
    mov8 r2 =OBJ_SIZE_FIELD_SIZE
    iadd
    mov8 =HEAP_END_PTR [r1]
    mov8 r2 =ADDRESS_SIZE
    iadd
    mov =HEAP_START r1

    # Check addr > heap_start && addr < heap_end && (addr - heap_start) % (obj_size + cell_meta_size) == 0
    #
    # addr > heap_start
    cmp =ADDR =HEAP_START
    jmple panic_free_out_of_heap

    # addr < heap_end
    cmp8 =ADDR =HEAP_END_PTR
    jmpge panic_free_out_of_heap

    # (addr - heap_start - cell_meta_size) % (obj_size + cell_meta_size) == 0
    mov r1 =ADDR
    mov r2 =HEAP_START
    isub
    dec r1
    mov =TMP_NORMALIZED_ADDR r1
    mov r1 =OBJ_SIZE
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


    !restore_reg_state r7
    !restore_reg_state r6
    !restore_reg_state r5
    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret


@ panic_free_out_of_heap
    !print_str free_out_of_heap_message
    !println_uint =ADDR
    mov1 error =OUT_OF_BOUNDS
    exit

@ panic_double_free
    !print_str double_free_message
    !println_uint =ADDR
    mov1 error =INVALID_INPUT
    exit

@ panic_misaligned_free
    !print_str misaligned_free_message
    !println_uint =ADDR
    mov1 error =INVALID_INPUT
    exit
