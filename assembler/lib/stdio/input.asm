# input.asm
# Export useful macros for getting user input from the console


.include:

    @@ interrupts.asm


.text:

    %% input_signed:

        intr [INPUT_SIGNED]

    %endmacro


    %% input_unsigned:

        intr [INPUT_UNSIGNED]

    %endmacro


    %% input_str:

        intr [INPUT_STRING]

    %endmacro

    