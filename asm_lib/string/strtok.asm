.include:

    "asmutils/functional.asm"
    "ctype/is_blank.asm"

.text:

# r1: string address
#
# Return:
#   -r1: token starting address
#   -r2: token end address
#
@@ strtok

    !save_reg_state r3
    !save_reg_state r4

    %- START: r3
    %- CURSOR: r4

    mov =START r1

    # Find token start (first non-blank token)
    @searching_start

        # Load the current character into r1
        mov1 r1 [=START]

        # A null termination means the token won't start at all
        cmp1 r1 '\0'
        jmpz end_search

        # Blank character means the token hasn't started yet
        call is_blank
        cmp1 r1 0
        jmpz end_search

        inc =START
        jmp searching_start

    @ end_search

    # Initialize the token cursor
    mov =CURSOR =START

    # Find the end of the token
    @tokenizing

        mov1 r1 [=CURSOR]

        # Null termination terminates the string
        cmp1 r1 '\0'
        jmpz end_tokenization

        # Blank character temrinates the token
        call is_blank
        cmp1 r1 0
        jmpnz end_tokenization

        inc =CURSOR
        jmp tokenizing

    @end_tokenization

    # Move the start of the token to r1 to return
    mov r1 =START
    # Move the end of the token to r2 to return
    mov r2 =CURSOR


    !restore_reg_state r4
    !restore_reg_state r3

    ret
