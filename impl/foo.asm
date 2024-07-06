.include:


.text:

    %% priv_macro arg:

        mov1 r1 {arg}
        !println_uint r1
        
    %endmacro

    %% pub_macro arg:

        !priv_macro {arg}

    %endmacro
