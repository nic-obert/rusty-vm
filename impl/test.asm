
.include:

    stdio.asm

.data:


.text:

@start

    jmp A
    @S1
        dd string "Hello\0"
    @A

    !println_str S1
    

