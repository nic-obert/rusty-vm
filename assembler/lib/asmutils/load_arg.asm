# load_arg
# Useful macros to load arguments from the stack


.text:

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
        mov r1 sbp
        isub

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
        mov r1 sbp
        isub

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
        mov r1 sbp
        isub

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
        mov r1 sbp
        isub

        mov8 {dest} [r1]
    
    %endmacro

    