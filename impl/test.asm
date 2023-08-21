
.include:

    string.asm
    stdio.asm

.data:

    A1 [char] ['a', 'b', 'c', 'd']
    A2 [char] ['m', 'n', 'o', 'p']


.text:

@start

    !println_bytes A1 4
    !println_bytes A2 4

    !println

    mov8 r1 A1
    mov8 r2 A2
    mov1 r3 4
    
    call memswap

    !println_bytes A1 4
    !println_bytes A2 4

