# time
# Time-related functions


.include:

    @@ interrupts.asm


.text:

    %% time_nanos:

        intr =HOST_TIME_NANOS

    %endmacro


    %% time_micros:

        intr =HOST_TIME_NANOS
        mov2 r2 1000
        idiv

    %endmacro


    %% time_millis:

        intr =HOST_TIME_NANOS
        mov4 r2 1000000
        idiv
    
    %endmacro


    %% time_secs:

        intr =HOST_TIME_NANOS
        mov4 r2 1000000000
        idiv

    %endmacro


    %% elapsed_nanos:

        intr =ELAPSED_TIME_NANOS

    %endmacro


    %% elapsed_micros:

        intr =ELAPSED_TIME_NANOS
        mov2 r2 1000
        idiv

    %endmacro


    %% elapsed_millis:

        intr =ELAPSED_TIME_NANOS
        mov4 r2 1000000
        idiv

    %endmacro


    %% elapsed_secs:

        intr =ELAPSED_TIME_NANOS
        mov4 r2 1000000000
        idiv

    %endmacro


