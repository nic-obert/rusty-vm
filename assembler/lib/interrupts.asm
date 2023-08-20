# interrupts
# Export named interrupt codes


.data:

    @@ PRINT_SIGNED u8 0
    @@ PRINT_UNSIGNED u8 1
    @@ PRINT_CHAR u8 2
    @@ PRINT_STRING u8 3
    @@ PRINT_BYTES u8 4
    
    @@ INPUT_SIGNED u8 5
    @@ INPUT_UNSIGNED u8 6
    @@ INPUT_STRING u8 7

    @@ MALLOC u8 8
    @@ FREE u8 9

