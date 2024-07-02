# single_ll
# Implementation of the singly-linked list data structure

# Sll memory structure:
#   - item_size: 8 bytes
#   - length: 8 bytes
#   - first: node* 8 bytes
#   - last: node* 8 bytes

# Node memory structure:
#   - next: node* 8 bytes
#   - item: `item_size` bytes


.include:

    "asmutils/functional.asm"
    "asmutils/pointer/ptr_def.asm"
    "stdlib/memory.asm"


.text:

    # Type-related definitions

    %- ITEM_SIZE_OFFSET: 0
    %- LENGTH_OFFSET: 8
    %- FIRST_OFFSET: 16
    %- LAST_OFFSET: 24

    %- NODE_NEXT_OFFSET: 0
    %- NODE_ITEM_OFFSET: 8

    %- NODE_STRUCT_BASE_SIZE: 8

    %%- SLL_STRUCT_SIZE: 32



    # Create a new singly-linked list on the stack
    #
    # Args:
    #   - item_size: 8 bytes
    #
    # Return:
    #   - tos: 
    #
    %% sll_new item_size:

        mov8 r1 {item_size}

        call sll_new

    %endmacro

    @@ sll_new 

        # Push the struct members in reverse order onto the stack

        # Last
        push8 =NULLPTR
        # First
        push8 =NULLPTR
        # Length
        push8 0
        # Item size
        push8 r1

        ret
    

    %% sll_append sll item_address:

        push8 {sll}
        push8 {item_address}

        call sll_append

        popsp1 16

    %endmacro

    @@ sll_append

        !set_fstart

        %- item_address: r8
        %- sll: r7

        !load_arg8 8 =item_address
        !load_arg8 16 =sll

        # Create a new node
        
        # Calculate the node size
        +

        ret


    % sll_node_size sll:

        
