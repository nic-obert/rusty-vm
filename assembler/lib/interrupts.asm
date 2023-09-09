# interrupts
# Export named interrupt codes


.text:

    %%- PRINT_SIGNED: 0
    %%- PRINT_UNSIGNED: 1
    %%- PRINT_CHAR: 2
    %%- PRINT_STRING: 3
    %%- PRINT_BYTES: 4
    
    %%- INPUT_SIGNED: 5
    %%- INPUT_UNSIGNED: 6
    %%- INPUT_STRING: 7

    %%- MALLOC: 8
    %%- FREE: 9

    %%- RANDOM: 10

    %%- HOST_TIME_NANOS: 11
    %%- ELAPSED_TIME_NANOS: 12

    %%- DISK_READ: 13
    %%- DISK_WRITE: 14

    %%- TERM_INTR: 15

    %%- SET_TIMER_NANOS: 16

    %%- FLUSH_STDOUT: 17

    %%- HOST_FS_INTR: 18

