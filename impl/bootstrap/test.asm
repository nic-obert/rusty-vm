.include:

    "stdio.asm"
    "string.asm"


.data:

    @program_start
    dcs "store v1 4"
    dcs "print hello world"
    dcs "exit"

    @exit_command
    dcs "exit"

.text:

    %- PC: r8

    mov8 =PC program_start

    @executing

        !println_str =PC

        # Interpret command
        mov r2 =PC
        mov8 r1 exit_command
        call strcmp
        cmp1 r1 1
        jmpz terminate

        # Update program counter
        !strlen =PC
        mov r2 =PC
        iadd
        inc r1
        mov =PC r1

        jmp executing

    @terminate

    exit
