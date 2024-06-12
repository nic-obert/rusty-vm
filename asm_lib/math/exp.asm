# Exponentiation functions


.include:

    constants.asm
    archlib.asm


.text:


    # r1: integer exponent
    # integer result in r1
    @@ expi2

        cmp1 r1 0
        jmpz zero_exp
        jmplt negative_exp

        %- counter: r3

        mov =counter r1
        mov1 r1 2

        @loop

            dec =counter
            jmpz endloop

            mov1 r2 2
            imul

            jmp loop
        
        @endloop

        ret

    
    @ negative_exp

        # Negative exponents are not allowed

        mov1 error =INVALID_INPUT
        ret

    
    @ zero_exp

        mov1 r1 0
        ret

