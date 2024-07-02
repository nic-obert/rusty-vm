# ascii_to_digit
# Convert an ASCII character to a u1 number
# Input: ASCII character stored in r1
# Output: u1 stored in r1
# If the conversion fails, an INVALID_INPUT error is set


.include:

    "errors.asm"


.text:

@@ ascii_to_digit

    # Check if the byte is in range 48..=57 (ASCII digit characters)

    cmp1 r1 48
    jmplt invalid
    
    cmp1 r1 57
    jmpgr invalid

    # The byte is a valid digit
    # Convert ASCII to integer
    mov1 r2 48
    isub

    # The result is stored in r1
    ret


@ invalid

    # Set the error code and return
    mov1 error =INVALID_INPUT
    ret

