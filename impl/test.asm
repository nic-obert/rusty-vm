.include:

    "stdio.asm"
    "allocators/object_allocator.asm"


.data:


.text:

    mov8 r1 8
    mov8 r2 1
    call init_object_allocator

    call alloc_object
    call alloc_object

    call free_object


    call free_object


   exit


@ok
    printstr "ok\n"
    ret
