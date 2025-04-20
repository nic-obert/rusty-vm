.include:

    "asmutils/functional.asm"
    "archlib.asm"

.data:

    @@ free_table_root
    dn 8 0

    @@ free_table_depth
    dn 8 0

    @@ heap_start
    dn 8 0

    @@ heap_end_ptr
    dn 8 0

    @@ pocket_allocator_handle
    dn 8 0


.text:

    %%- NODE_TYPE_FREE_LEAF: 0
    %%- NODE_TYPE_PARENT: 1
    %%- NODE_TYPE_OCCUPIED_LEAF: 2

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


# Args:
#   - r1: node ptr
#
# Returns:
#   - r1: node type
#
@@ get_node_type

    mov1 r1 [r1]

    ret


# Args:
#   - r1: node ptr
#
# Returns:
#   - r1: left child
#
# Warning: undefined behavior if the node doesn't have children
#
@@ get_left_child

    inc r1
    mov8 r1 [r1]

    ret


# Args:
#   - r1: node ptr
#
# Returns:
#   - r1: left child
#
# Warning: undefined behavior if the node doesn't have children
#
@@ get_right_child

    !save_reg_state r2

    mov8 r2 =RIGHT_CHILD_OFFSET
    iadd
    mov8 r1 [r1]

    !restore_reg_state r2

    ret


# Args:
#   - r1: node ptr
#
@@ set_node_occupied

    mov1 [r1] =NODE_TYPE_OCCUPIED_LEAF

    ret
