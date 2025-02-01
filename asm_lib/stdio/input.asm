.include:

    "archlib.asm"
    "asmutils/functional.asm"

.text:

# r1: buf address
# r2: buf size
#
# Read bytes from stdin and copy them into the buffer
#
@@read_buf

    mov1 int =INPUT_STRING
    intr

    ret


# r1: buf address
# r2: buf size
#
# Copy a string from stdin into the buffer and add a null termination.
# Assume the buffer is large enough for the null-terminated string
#
@@read_str
    !save_reg_state r1
    !save_reg_state r2

    call read_buf

    # bytes read is now in input register

    mov r2 input
    # decrement the size to overwrite the newline
    dec r2
    # calculate the end of the string
    iadd
    # r1: end of string
    # null-terminate the string
    mov1 [r1] '\0'

    !restore_reg_state r2
    !restore_reg_state r1

    ret
