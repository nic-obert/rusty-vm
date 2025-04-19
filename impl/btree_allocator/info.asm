.include:

    "shared.asm"

.text:

# Returns:
#   - r1: heap size
@@ btree_heap_size

    !save_reg_state r2

    mov8 r1 [heap_start]
    mov8 r2 [heap_end_ptr]
    isub

    !restore_reg_state r2

    ret
