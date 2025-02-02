.include:

    "archlib.asm"
    "asmutils/functional.asm"
    "asmutils/error_handling.asm"


.text:

%% read_byte:

    mov1 int =INPUT_BYTE
    intr

%endmacro


# r1: buf address
# r2: buf size
#
# Read bytes from stdin and copy them into the buffer
#
@@ read_buf

    mov1 int =INPUT_STRING
    intr

    ret


# r1: buf address
# r2: buf size
#
# Copy a string from stdin into the buffer and add a null termination.
# Assume the buffer is large enough for the null-terminated string
#
@@ read_str

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


# r1: buf address
# r2: buf size > 1
#
# Copy the first word from stdin into the buffer and add a null termination.
#
@@ read_word

    %- WORD_LEN: r3
    %- BUF_ADDRESS: r4
    %- BUF_SIZE: r5

    # Enforce BUF_SIZE > 1
    cmp8 r2 1
    jmpgr buf_size_ok
        mov1 error =INVALID_INPUT
        ret
    @buf_size_ok

    !save_reg_state r1
    !save_reg_state r2
    !save_reg_state r3
    !save_reg_state r4
    !save_reg_state r5


    mov8 =WORD_LEN 0
    mov =BUF_ADDRESS r1
    mov =BUF_SIZE r2

    @get_chars

        !read_byte

        # Break if errors were encountered, including EOF
        !jmperr end_get_chars

        # Break the loop when a space character is encountered
        cmp1 input ' '
        jmpz end_get_chars
        cmp1 input '\t'
        jmpz end_get_chars
        cmp1 input '\n'
        jmpz end_get_chars

        # Append the current char to the word
        mov r1 =BUF_ADDRESS
        mov r2 =WORD_LEN
        iadd
        mov1 [r1] input

        inc =WORD_LEN

        # Continue while WORD_LEN+1 < BUF_SIZE
        mov r1 =WORD_LEN
        inc r1
        cmp r1 =BUF_SIZE
        jmplt get_chars
    @end_get_chars

    # null-terminate the word
    mov r1 =BUF_ADDRESS
    mov r2 =WORD_LEN
    iadd
    mov1 [r1] '\0'


    !restore_reg_state r5
    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret


%% stdin_has_data:

    mov1 int =STDIN_HAS_DATA
    intr

%endmacro


@@ flush_stdin

    !save_reg_state r1

    @flushing

        !stdin_has_data
        cmp8 r1 0
        jmpz finish_flushing

        !read_byte
        !jmpnoerr flushing

    @finish_flushing

    !restore_reg_state r1

    ret
