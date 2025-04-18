.include:

    "lib.asm"

    "stdio.asm"


.data:

    @alloc_countdown_msg
    dcs "Allocations remaining: "

.text:

    mov8 r1 8
    mov8 r2 16
    call init_pocket_object_allocator
    mov r8 r1

    mov r1 r8
    call alloc_object

    !println_uint r8
    !println_uint r1
    !println_uint pep

    mov r2 r1
    mov r1 r8
    call free_object

    mov8 r7 16

    @loop

        !print_str alloc_countdown_msg
        !println_uint r7
        mov r1 r8
        call alloc_object

        dec r7
        jmpnz loop


    exit
