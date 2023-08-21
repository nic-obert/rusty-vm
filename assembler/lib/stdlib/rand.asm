# rand
# Random number generation


.include:

    @@ interrupts.asm


.text:

    %% rand:

        intr [RAND]
    
    %endmacro

