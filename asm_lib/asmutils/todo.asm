.include:

    "stdio.asm"
    "archlib.asm"

.data:

    @@TODO_EXIT_MESSAGE
    ds "TODO encountered: aborting.\0"

.text:

    %% todo:

        !println_str TODO_EXIT_MESSAGE
        mov1 error =INTERRUPTED
        exit

    %endmacro
