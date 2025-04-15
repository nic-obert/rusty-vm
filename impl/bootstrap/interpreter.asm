.include:

    "allocators/object_allocator.asm"
    "stdio.asm"


.data:


.text:

    # A shell-like interpreter

    # Object size
    mov8 r1 8
    # Max objects
    mov8 r2 4
    call init_object_allocator


    exit
