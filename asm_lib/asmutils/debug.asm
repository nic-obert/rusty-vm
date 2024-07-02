.include:

    "archlib.asm"
    "stdio/print.asm"


.text:

    %% breakpoint:

        intr =INPUT_SIGNED

    %endmacro


    %% breakpoint_msg msg:

        !print_str {msg}
        intr =INPUT_SIGNED

    %endmacro

