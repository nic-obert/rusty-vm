# time
# Time-related functions


.include:

    "archlib.asm"


.text:

    %% time_nanos:

        mov1 int =HOST_TIME_NANOS
        intr

    %endmacro


    %% time_micros:

        mov1 int =HOST_TIME_NANOS
        intr
        mov2 r2 1000
        idiv

    %endmacro


    %% time_millis:

        mov1 int =HOST_TIME_NANOS
        intr
        mov4 r2 1000000
        idiv

    %endmacro


    %% time_secs:

        mov1 int =HOST_TIME_NANOS
        intr
        mov4 r2 1000000000
        idiv

    %endmacro


    %% elapsed_nanos:

        mov1 int =ELAPSED_TIME_NANOS
        intr

    %endmacro


    %% elapsed_micros:

        mov1 int =HOST_TIME_NANOS
        intr
        mov2 r2 1000
        idiv

    %endmacro


    %% elapsed_millis:

        mov1 int =HOST_TIME_NANOS
        intr
        mov4 r2 1000000
        idiv

    %endmacro


    %% elapsed_secs:

        mov1 int =HOST_TIME_NANOS
        intr
        mov4 r2 1000000000
        idiv

    %endmacro


    %% set_timer_nanos t:

        mov8 r1 {t}
        mov1 int =SET_TIMER_NANOS
        intr

    %endmacro


    %% set_timer_micros t:

        mov8 r1 {t}
        mov2 r2 1000
        imul
        mov1 int =SET_TIMER_NANOS
        intr

    %endmacro


    %% set_timer_millis t:

        mov8 r1 {t}
        mov4 r2 1000000
        imul
        mov1 int =SET_TIMER_NANOS
        intr

    %endmacro


    %% set_timer_secs t:

        mov8 r1 {t}
        mov4 r2 1000000000
        imul
        mov1 int =SET_TIMER_NANOS
        intr

    %endmacro
