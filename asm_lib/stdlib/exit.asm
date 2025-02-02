# exit
# Macro to exit with the given exit code


.text:

    %% _exit code:

        mov1 error {code}
        exit

    %endmacro
