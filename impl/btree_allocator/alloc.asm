.include:

    "shared.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "stdio/print.asm"

.data:

    @out_of_memory_message
    dcs "The allocator is out of memory"

    @zero_size_message
    dcs "Requested allocation of zero bytes"

    @malformed_node_message
    dcs "Free table node was malformed"

.text:

# Args
#   - r1: non-zero requested size
#
# Return:
#   - r1: non-null allocated block
#
# Panics if out of memory or if the requested size is zero
#
@@ btree_alloc

    # Check for zero-sized allocation requests
    cmp8 r1 0
    jmpz panic_zero_size

    !save_reg_state r8

    %- REQUESTED_SIZE: r8

    mov =REQUESTED_SIZE r1

    # Search for a block of the requested size
    #
    mov8 r1 [free_table_root]
    mov8 r1 0
    call alloc_recursive



    !restore_reg_state r8

    ret


# Args:
#   - r1: starting node
#   - r2: current depth
#   - r8: requested size (is kept in r8 through the whole subroutine)
#
# Returns:
#   - r1: 0 if the node is occupied, the node address if it's free
#
# Panics if a malformed node is encountered
#
@ alloc_recursive

    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4

    %- CURRENT_NODE: r3
    %- CURRENT_DEPTH: r4

    mov =CURRENT_NODE r1
    mov =CURRENT_DEPTH r2

    # TODO: verify the requested size <= node size

    # Match node type
    #
    call get_node_type
    cmp1 r1 =NODE_TYPE_FREE_LEAF
    jmpz node_type_free_leaf
    cmp1 r1 =NODE_TYPE_OCCUPIED_LEAF
    jmpz node_type_occupied_leaf
    cmp1 r1 =NODE_TYPE_PARENT
    jmpz node_type_parent

    call panic_malformed_node

    # Case handlers
    #
    @node_type_free_leaf

        # This is a free leaf, calculate whether to split the node or keep it intact
        # TODO

    jmp after_node_match

    @node_type_occupied_leaf

        # The node is a leaf node and is occupied
        # Return null
        mov8 r1 0

    jmp after_node_match

    @node_type_parent

        # Check each child node
        #
        inc =CURRENT_DEPTH
        mov r2 =CURRENT_DEPTH

        # Try with the left node
        mov r1 =CURRENT_NODE
        call get_left_child
        call alloc_recursive
        cmp8 r1 0
        jmpnz after_node_match # Free node was found return it

        # Try with the right node
        mov r1 =CURRENT_NODE
        call get_right_child
        call alloc_recursive

    @after_node_match
    #
    # End of case handlers

    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2

    ret


@ panic_zero_size
    !println_str zero_size_message
    mov1 error =INVALID_INPUT
    exit

@ panic_out_of_memory
    !println_str out_of_memory_message
    mov1 error =OUT_OF_MEMORY
    exit

@ panic_malformed_node
    !println_str malformed_node_message
    mov1 error =INVALID_DATA
    exit
