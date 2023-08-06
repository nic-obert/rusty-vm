
.data:

    s string "Jumping\n"

.text:

@start

    mov8 print s
    printstr        

    jmp start

