# is_digit
# Check if an ASCII character is a digit
# Input: ASCII character in r1
# Output in r1: 1 if char is digit, else 0


.text:

@@ is_digit

    # Check if the byte is in range 48..=57 (ASCII digit characters)

    cmp1 r1 48
    jmplt invalid
    
    cmp1 r1 57
    jmpgr invalid

    # Char is a digit, return 1
    mov1 r1 1
    ret


@ invalid

    # Char is not a digit, return 0
    mov1 r1 0
    ret

