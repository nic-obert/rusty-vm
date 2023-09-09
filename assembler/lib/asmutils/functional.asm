# functional
# Useful macros to implement the self-contained function pattern


.text:

    # Use a non-standard register to store the start of the function's stack
    #
    %- FSTART_REG: input


    # Set the start of the function stack.
    # This macro should be called as the first instruction of a function to avoid changing the stack pointer.
    #
    %% set_fstart:

        mov =FSTART_REG sbp

    %endmacro


    # Save all the current general purpose register states by pushing them onto the stack
    #
    %% save_all_reg_states:

        push8 r1
        push8 r2
        push8 r3
        push8 r4
        push8 r5
        push8 r6
        push8 r7
        push8 r8

    %endmacro


    # Restore all the previous general purpose register states by popping them from the stack
    #
    %% restore_all_reg_states:

        pop8 r8
        pop8 r7
        pop8 r6
        pop8 r5
        pop8 r4
        pop8 r3
        pop8 r2
        pop8 r1

    %endmacro


    # Save the current register state by pushing it onto the stack
    #
    %% save_reg_state reg:

        push8 {reg}

    %endmacro


    # Restore the previous register state by popping it from the stack
    #
    %% restore_reg_state reg:

        pop8 {reg}

    %endmacro

    
    # Load a 1-byte argument
    #
    # Args:
    #   - offset: 8-byte positive offset
    #   - dest: generic destination to store the argument
    #
    # Registers used:
    #   - r1
    #   - r2
    #
    %% load_arg1 offset dest:

        # Calculate the argument stack address
        mov8 r2 {offset}
        mov r1 =FSTART_REG
        iadd

        mov1 {dest} [r1]
    
    %endmacro


    # Load a 2-byte argument
    #
    # Args:
    #   - offset: 8-byte positive offset
    #   - dest: generic destination to store the argument
    #
    # Registers used:
    #   - r1
    #   - r2
    #
    %% load_arg2 offset dest:

        # Calculate the argument stack address
        mov8 r2 {offset}
        mov r1 =FSTART_REG
        iadd

        mov2 {dest} [r1]
    
    %endmacro


    # Load a 4-byte argument
    #
    # Args:
    #   - offset: 8-byte positive offset
    #   - dest: generic destination to store the argument
    #
    # Registers used:
    #   - r1
    #   - r2
    #
    %% load_arg4 offset dest:

        # Calculate the argument stack address
        mov8 r2 {offset}
        mov r1 =FSTART_REG
        iadd

        mov4 {dest} [r1]
    
    %endmacro


    # Load a 8-byte argument
    #
    # Args:
    #   - offset: 8-byte positive offset
    #   - dest: generic destination to store the argument
    #
    # Registers used:
    #   - r1
    #   - r2
    #
    %% load_arg8 offset dest:

        # Calculate the argument stack address
        mov8 r2 {offset}
        mov r1 =FSTART_REG
        iadd

        mov8 {dest} [r1]
    
    %endmacro

