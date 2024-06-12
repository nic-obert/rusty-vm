# is_alphanumeric
# Check if an ASCII character is alphanumeric
# r1: input ASCII character
# Output: set r1 to 1 if char is alphanumeric, else 0


.text:

    @@ is_alphanumeric

        cmp1 r1 '0'
        jmplt invalid

        cmp1 r1 '9'
        jmple valid

        cmp1 r1 'A'
        jmplt invalid

        cmp1 r1 'Z'
        jmple valid

        cmp1 r1 'a'
        jmplt invalid

        cmp1 r1 'z'
        jmple valid


    @invalid

        # Char is not alphanumeric, return 0
        
        mov1 r1 0
        ret

    
    @valid

        # Char is alphanumeric, return 1

        mov1 r1 1
        ret

