# is_blank.asm
# Return whether char in r1 is a blank character
# Return value in r1 is 1 if char is a blank character, else 0


.text:

    @@ is_space

        cmp1 r1 ' '
        jmpz char_is_space

        cmp1 r1 '\t'
        jmpz char_is_space

        mov1 r1 0
        ret

    @ char_is_space

        mov1 r1 1
        ret
        
        