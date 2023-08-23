# seconds counter
# Count the seconds since the program has started and print them to the console.


.data:

    OS string "Seconds elapsed: \0"


.include: 

    stdio.asm
    time.asm


.text:

@start

    # Initialize the last time register
    mov1 r8 0

    !println

    @loop

        !elapsed_secs

        cmp r1 r8
        jmpz loop

        !print_char '\r'
        !print_str OS
        !print_uint r1

        mov r8 r1

        jmp loop

