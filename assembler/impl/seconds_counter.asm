# seconds counter
# Count the seconds since the program has started and print them to the console.


.data:

    OS string "Seconds elapsed: \0"


.include: 

    stdio.asm
    time.asm


.text:

@start

    %- last_time: r8

    # Initialize the last time register
    mov1 =last_time 0

    !println

    @loop

        !elapsed_secs

        cmp r1 =last_time
        jmpz loop

        !print_char '\r'
        !print_str OS
        !print_uint r1

        mov =last_time r1

        jmp loop

