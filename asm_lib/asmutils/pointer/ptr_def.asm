# ptr_def
# Useful definitions for pointers


.text:

    %%- NULLPTR: 0xffffffffffffffff


    %% ptr_is_null ptr:

        cmp8 {ptr} =NULLPTR
    
    %endmacro
