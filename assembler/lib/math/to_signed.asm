# to_signed
# Convert positive integer to its two's complement negative counterpart
# Store the result in r1


.text:

    %% to_signed num:

        mov8 r2 {num}
        mov1 r1 0
        isub
    
    %endmacro

