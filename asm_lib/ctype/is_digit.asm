# is_digit
# Check if an ASCII character is a digit
# Input: ASCII character in r1
# Output in r1: 1 if char is digit, else 0


.text:

@@ is_digit

    cmp1 r1 '0'
    jmplt invalid
    
    cmp1 r1 '9'
    jmpgr invalid

    # Char is a digit, return 1

    mov1 r1 1
    ret


@ invalid

    # Char is not a digit, return 0

    mov1 r1 0
    ret

