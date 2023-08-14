# is_space.asm
# Return whether char in r1 is a whitespace
# Return value in r1 is 1 if char is a whitespace, else 0


.text:

@@ is_space

    cmp1 r1 ' '
    jmpz char_is_space

    cmp1 r1 '\t'
    jmpz char_is_space

    cmp1 r1 '\n'
    jmpz char_is_space

    cmp1 r1 '\r'
    jmpz char_is_space

    mov1 r1 0
    ret

    @ char_is_space

    mov1 r1 1
    ret
    
    