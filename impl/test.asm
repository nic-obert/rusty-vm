
.include:

    stdio/print.asm
    asmutils/static_def.asm

.data:


.text:

@start

    !static_def str string "Hello World!\0"

    !println_str str

    mov r1 =boh

    exit

