.text:

    %% array_get_item_size_ptr array:

        mov r1 {array}
    
    %endmacro


    %% array_get_length_ptr array:

        mov r1 {array}
        mov1 r2 =ARRAY_LENGTH_OFFSET
        iadd

    %endmacro


    %% array_get_data_ptr array:

        mov r1 {array}
        mov1 r2 =ARRAY_DATA_OFFSET
        iadd

    %endmacro

