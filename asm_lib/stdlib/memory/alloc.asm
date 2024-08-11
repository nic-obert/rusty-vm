
# To create a complex allocator, it would be nice to build it from the ground up through simpler steps.
# A buddy allocator is a good option.
# The buddy allocator may use a tree (probably a binary tree) internally
# Such a binary tree would need an allocator of its own. 
# Since the binary tree is comprised of simple nodes, a fixed-size allocator or a slab allocator may be used
# to implement tree. 
# However, the allocator would be better implemented not in assembly, but in a higher-level language like oxide
# This means that I should finish the oxide compiler and get it to a working state as soon as possible
# Coding an allocator in assembly is too much prone to errors and would be quite frustrating


.include:

    "stdlib/exit.asm"
    "stdio/print.asm"
    "stdlib/memory/set_zeros.asm"


.alloc_internal:

    # The base address of the allocator system
    @alloc_base
    dn 8 0

    # The allocated size
    @alloc_size
    dn 8 0


.text:


    %%- ALLOC_CHUNK_METADATA_SIZE: 8


    # Initialize the allocator system.
    # This macro should be called as the first instruction, outside any poppable stack frame.
    #
    # Args:
    #   - size: size to allocate (8 bytes)
    #
    %% alloc_init size:

        mov8 [alloc_base] stp
        mov8 [alloc_size] {size}


        # Make space for the heap and store its base address

        mov8 [alloc_base] stp
        pushsp8 [alloc_size]


        # Initialize the heap to a blank state
        # This is necessary beacuse the allocator uses metatata attached to allocated memory blocks

        mov8 r1 [alloc_base]
        mov8 r2 [alloc_size] 
        call set_zeros

    %endmacro



    # size in r1
    # return in r1
    @@ malloc

        # Iterate through all the memory blocks until a free block is found

        ret


    %% free ptr:



    %endmacro

    @@ free


        ret


 