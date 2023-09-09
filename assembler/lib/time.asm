# time
# Time-related functions


.include:

    interrupts.asm


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


    %% set_timer_nanos t:

        mov8 r1 {t}
        intr =SET_TIMER_NANOS

    %endmacro


    %% set_timer_micros t:

        mov8 r1 {t}
        mov2 r2 1000
        imul
        intr =SET_TIMER_NANOS
    
    %endmacro


    %% set_timer_millis t:

        mov8 r1 {t}
        mov4 r2 1000000
        imul
        intr =SET_TIMER_NANOS
    
    %endmacro


    %% set_timer_secs t:

        mov8 r1 {t}
        mov4 r2 1000000000
        imul
        intr =SET_TIMER_NANOS
    
    %endmacro

