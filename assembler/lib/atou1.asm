# atou1
# Convert an ASCII character to a u1 number
# Input: ASCII character stored in r1
# Output: u1 stored in r1
# If the conversion fails, an INVALID_INPUT error is set


.text:

@error

    # Set the error code and return
    mov1 error INVALID_INPUT
    ret

@@ atou1

    # Check if the byte is in range 48..58 (ASCII digit characters)
    cmp8 r1 48
