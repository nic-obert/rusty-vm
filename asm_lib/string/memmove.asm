# memmove


.include:

    "string/memcpy.asm"
    "asmutils/functional.asm"


.text:

    # Copy `num` bytes from `src` into `dest`, checking overlapping memory regions.
    # For copying unsafely overlapping memory regions, implements a safe copy using an intermediate buffer.
    #
    # Args:
    #   - src: source memory region address (8 bytes)
    #   - dest: destination memory region address (8 bytes)
    #   - num: number of bytes to copy (8 bytes)
    #
    %% memmove src dest num:

        push8 {src}
        push8 {dest}
        push8 (num)

        call memmove

        popsp 24

    %endmacro

    @@ memmove

        !set_fstart

        !save_reg_state r1
        !save_reg_state r2
        !save_reg_state r3
        !save_reg_state r4
        !save_reg_state r5


        %- src: r4
        %- dest: r5
        %- num: r3

        !load_arg8 8 =num
        !load_arg8 16 =dest
        !load_arg8 24 =src


        # Check if source and destination regions unsafely overlap
        # Unsafe overlap configuration is (src < dest && src + num > dest)
        # Other overlapping configurations are safe for memcpy

        # Filter out non-overlapping and safely overlapping configurations

        # src < dest
        cmp =src =dest
        jmpge no_overlap

        mov r1 =src
        mov r2 =num
        iadd

        # src + num > dest
        cmp r1 =dest
        jmple no_overlap


    # Unsafe overlap case

        # Allocate the intermediate buffer of `num` bytes on the stack
        pushsp =num

        # Copy source into intermediate buffer on the stack
        !memcpy =src stp =num

        # Copy intermediate buffer into destination
        !memcpy stp =dest =num

        # Pop the intermediate buffer after it's being used
        popsp =num

        ret


    @ no_overlap

    !memcpy =src =dest =num

    !restore_reg_state r5
    !restore_reg_state r4
    !restore_reg_state r3
    !restore_reg_state r2
    !restore_reg_state r1

    ret

