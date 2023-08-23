# error_handling
# Useful library for handling errors


.include:

    @@ errors.asm


.text:

    # Jump to the specified label if the error register is set (error != 0)
    %% jmperr label:

        cmp1 error =NO_ERROR
        jmpnz {label}

    %endmacro


    # Jump to the specified label if the error register is not set (error == 0)
    %% jmpnoerr label:

        cmp1 error =NO_ERROR
        jmpz {label}

    %endmacro

