.include:

    "shared.asm"
    "asmutils/functional.asm"
    "allocators/pocket_object_allocator.asm"
    "math/pow.asm"

.data:

.text:

# Args:
#   - r1: minimum heap size (will be normalized to heap_size = MIN_BLOCK_SIZE * 2^n where n is a positive integer)
#   - pep: heap buffer start
#
# Returns:
#   - pep: heap buffer end
#
@@ init_btree_allocator

    !save_reg_state r1
    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4

    %- TREE_MAX_DEPTH: r3
    %- HEAP_SIZE: r4

    call validate_heap_size
    mov =HEAP_SIZE r1
    mov =TREE_MAX_DEPTH r2
    mov8 [free_table_depth] =TREE_MAX_DEPTH

    # Allocate the heap buffer
    #
    mov8 [heap_start] pep
    mov r1 pep
    mov r2 =HEAP_SIZE
    iadd
    mov pep r1
    mov [heap_end_ptr] r1

    # Initialize the pocket allocator for the free table
    #
    # Calculate the node count = `2^D - 1` where D is the depth of the tree
    mov r1 =TREE_MAX_DEPTH
    call pow2
    dec r1
    # Create the pocket allocator
    mov r2 r1
    mov8 r1 =NODE_SIZE
    call init_pocket_allocator
    mov8 [pocket_allocator_handle] r1

    # Initialize the free table
    #
    # Create the root node
    mov8 r1 [pocket_allocator_handle]
    call pocket_alloc_object
    # Initialize the root node
    call init_root_node
    # Store the root node handle
    mov8 [free_table_root] r1


    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret


# Args:
#   - r1: minimum heap size S
#
# Returns:
#   - r1: normalized heap heap size
#   - r2: n-1 value in `S <= MIN_BLOCK_SIZE * 2^n`
#         This value will also be the depth of the btree
#
@ validate_heap_size

    !save_reg_state r3
    !save_reg_state r4

    %- MIN_SIZE: r3
    %- n: r4

    mov =MIN_SIZE r1
    mov8 r1 =MIN_BLOCK_SIZE
    mov8 r2 2
    mov8 =n 1

    @pow_loop

        cmp r1 =MIN_SIZE
        jmpge end_pow_loop
        imul
        inc =n

        jmp pow_loop
    @end_pow_loop

    mov r2 =n

    !restore_reg_state r4
    !restore_reg_state r3

    ret
