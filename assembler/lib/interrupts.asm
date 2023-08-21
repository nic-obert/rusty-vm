# interrupts
# Export named interrupt codes


.data:

    @@ PRINT_SIGNED u1 0
    @@ PRINT_UNSIGNED u1 1
    @@ PRINT_CHAR u1 2
    @@ PRINT_STRING u1 3
    @@ PRINT_BYTES u1 4
    
    @@ INPUT_SIGNED u1 5
    @@ INPUT_UNSIGNED u1 6
    @@ INPUT_STRING u1 7

    @@ MALLOC u1 8
    @@ FREE u1 9

    @@ RANDOM u1 10

    @@ HOST_TIME_NANOS u1 11
    @@ ELAPSED_TIME_NANOS u1 12

