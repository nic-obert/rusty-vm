.include:

    "archlib.asm"
    "asmutils/functional.asm"
    "stdbool.asm"


.text:

    # Read `size` bytes from local storage at `address` into `buffer`
    #
    # Args:
    #   - address: the disk read address (8 bytes)
    #   - buffer: the buffer address to read into (8 bytes)
    #   - size: the number of bytes to read (8 bytes)
    #
    %% disk_read address buffer size:

        push8 {address}
        push8 {buffer}
        push8 {size}

        call disk_read

        popsp1 24

    %endmacro
    
    @@ disk_read

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3

        %- address: r1
        %- buffer: r2
        %- size: r3

        !load_arg8 8 =size
        !load_arg8 16 =buffer
        !load_arg8 24 =address

        intr =DISK_READ

        !restore_reg_state r3
        !restore_reg_state r2
        !restore_reg_state r1

        ret

    
    # Write `size` bytes from `buffer` into local storage at `address`
    #
    # Args:
    #   - address: the disk write address (8 bytes)
    #   - buffer: the buffer address to write (8 bytes)
    #   - size: the number of bytes to write (8 bytes)
    #
    %% write_disk address buffer size:

        push8 {address}
        push8 {buffer}
        push8 {size}

        call write_disk

        popsp1 24

    %endmacro

    @@ disk_write

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3

        %- address: r1
        %- buffer: r2
        %- size: r3

        !load_arg8 8 =size
        !load_arg8 16 =buffer
        !load_arg8 24 =address

        intr =DISK_WRITE

        !restore_reg_state r3
        !restore_reg_state r2
        !restore_reg_state r1

        ret
    

    # Check if a disk is available
    #
    # Return:
    #   - r1: 1 if a disk is available, 0 otherwise
    #
    @@ has_disk

        # Dummy read, fails if no disk is available
        !disk_read 0 0 0

        cmp1 error =MODULE_UNAVAILABLE
        !bool_invert_zf
        mov r1 zf

        ret

