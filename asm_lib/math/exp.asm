# Exponentiation functions


.include:

    "constants.asm"
    "archlib.asm"
    "asmutils/functional.asm"
    "stdio.asm"


.text:


    # r1: integer exponent
    # result stored in r1
    @@ expi

        !save_reg_state r3

        cmp1 r1 0
        jmpz zero_exp
        jmplt negative_exp

        %- counter: r3

        mov =counter r1
        mov8 r1 =e

        @expi_loop

            dec =counter
            jmpnz endloop_expi

            mov8 r2 =e
            fmul

            jmp expi_loop

        @endloop_expi

        !restore_reg_state r3

        ret


    # r1: integer exponent
    # integer result in r1
    @@ expi2

        !save_reg_state r3

        cmp1 r1 0
        jmpz zero_exp
        jmplt negative_exp

        %- counter: r3

        mov =counter r1
        mov1 r1 2

        @expi2_loop

            dec =counter
            jmpz endloop_expi2

            mov1 r2 2
            imul

            jmp expi2_loop
        
        @endloop_expi2

        ret

    
    @ negative_exp

        # Negative exponents are not allowed

        mov1 error =INVALID_INPUT

        !restore_reg_state r3

        ret

    
    @ zero_exp

        mov1 r1 0

        !restore_reg_state r3

        ret

