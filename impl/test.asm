.include:


.data:



.text:

    mov8 r1 0

    @loop
        inc r1
        cmp8 r1 100000
        jmplt loop

    breakpoint

    exit
