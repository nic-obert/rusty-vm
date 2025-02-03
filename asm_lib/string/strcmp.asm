# strcmp


.include:

    "asmutils/functional.asm"


.text:

# Compare two null-terminated strings and return whether they are equal
#
# Args:
#   - r1: the first string address
#   - r2: the second string address
#
# Return:
#   - r1: 1 if the strings are equal, 0 otherwise
#
@@ strcmp

    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4

    %- RES: r2
    %- S1: r3
    %- S2: r4

    mov =S1 r1
    mov =S2 r2
    mov1 =RES 0

    @checking

        # Check if characters are equal
        cmp1 [=S1] [=S2]
        jmpnz not_equal

        # Chars are equal. Check if string is terminated
        cmp1 [=S1] '\0'
        jmpz equal

        inc =S1
        inc =S2

        jmp checking

    @equal
        mov1 =RES 1
    @not_equal

    mov r1 =RES

    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2

    ret
