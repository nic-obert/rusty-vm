.include:

    "stdio.asm"
    "allocators/object_allocator.asm"


.data:


.text:

    mov8 r1 -9
    mov8 r2 -1
    isub

   exit


@ok
    printstr "ok\n"
    ret
