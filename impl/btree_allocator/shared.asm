.include:

    "asmutils/functional.asm"
    "archlib.asm"

.data:

    @@ free_table_root
    dn 8 0

    @@ heap_start
    dn 8 0

    @@ heap_end_ptr
    dn 8 0

    @@ pocket_allocator_handle
    dn 8 0


.text:

    %%- NODE_TYPE_FREE_LEAF: 0
    %%- NODE_TYPE_FREE_PARENT: 1
    %%- NODE_TYPE_OCCUPIED_PARENT: 2
    %%- NODE_TYPE_OCCUPIED_LEAF: 3

    %%- NODE_TYPE_OFFSET: 0
    %%- LEFT_CHILD_OFFSET: 1
    %%- RIGHT_CHILD_OFFSET: 9
    %%- NODE_SIZE: 17

    %%- MIN_BLOCK_SIZE: 8


# Args:
#   - r1: uninitialized root node ptr
#
@@ init_root_node

    !save_reg_state r1
    !save_reg_state r2

    # Initialize the node type to FREE LEAF
    mov1 [r1] =NODE_TYPE_FREE_LEAF
    inc r1
    # Initialize the children pointers to null
    mov8 [r1] 0
    mov8 r2 =ADDRESS_SIZE
    iadd
    mov8 [r1] 0

    !restore_reg_state r2
    !restore_reg_state r1

    ret
