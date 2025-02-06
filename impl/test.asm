.include:

    "stdio.asm"
    "allocators/object_allocator.asm"


.data:


.text:

    mov8 r1 1
    mov8 r2 18446744073709551615
    isub

   exit


@ok
    printstr "ok\n"
    ret
