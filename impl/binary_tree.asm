# Warning: a single binary tree instance can be present at any time
# The global object allocator is overwritten

# Node structure
# - value: 8 bytes
# - child 1: 8 bytes nullable ptr
# - child 2: 8 bytes nullable ptr
#

.include:

    "asmutils/functional.asm"
    "allocators/object_allocator.asm"
    "math/pow.asm"

    "stdio.asm"


.data:


.text:

    %- NODE_DATA_SIZE: 8
    %- NODE_CHILD1_OFFSET: 8
    %- NODE_CHILD2_OFFSET: 16
    %- NODE_SIZE: 24

    mov8 r1 4
    call init_binary_tree


    exit


# Args:
# - r1: max_depth
#
# Return:
# - r1: tree root node
#
@ init_binary_tree

    !save_reg_state r1
    !save_reg_state r2

    # Calculate the node count: 2^depth -1
    # r1 is already max_depth
    call pow2
    dec r1
    # r1 is now the node count

    mov r2 r1
    mov8 r1 =NODE_SIZE
    call init_object_allocator

    call alloc_object

    # Initialize the root node
    # Leave the node value uninitialized
    # Initialize the children to null
    mov8 r2 0
    call node_set_child1
    call node_set_child2

    !restore_reg_state r2
    !restore_reg_state r1

    ret


# Args:
#   - r1: node address
#   - r2: value
#
@ node_set_value

    mov8 [r1] r2

    ret


# Args:
#   - r1: node address
#   - r2: new child 1 address
#
@ node_set_child1

    !save_reg_state r1
    !save_reg_state r2
    !save_reg_state r3

    # Save new child address
    mov r3 r2

    # Calculate field address
    mov8 r2 =NODE_CHILD1_OFFSET
    iadd

    # Set the child 1 field
    mov8 [r1] r3

    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret


# Args:
#   - r1: node address
#   - r2: new child 2 address
@ node_set_child2

    !save_reg_state r1
    !save_reg_state r2
    !save_reg_state r3

    # Save new child address
    mov r3 r2

    # Calculate field address
    mov8 r2 =NODE_CHILD2_OFFSET
    iadd

    # Set the child 2 field
    mov8 [r1] r3

    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret
