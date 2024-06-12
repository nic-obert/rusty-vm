# is_alpha
# Check if an ASCII character is alphabetic
# r1: input ASCII character
# Output: set r1 to 1 if char is alphabetic, else 0


.text:

    @@ is_alpha

        cmp1 r1 'A'
        jmplt invalid

        cmp1 r1 'Z'
        jmple valid

        cmp1 r1 'a'
        jmplt invalid

        cmp1 r1 'z'
        jmple valid


    @invalid

        # Char is not alphabetic, return 0

        mov1 r1 0
        ret

    
    @valid

        # Char is alphabetic

        mov1 r1 1
        ret

