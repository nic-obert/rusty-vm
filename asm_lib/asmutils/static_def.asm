# static_def


.text:

    # Define static local data of the given type and value.
    # Take care of adding a constant jump instruction to prevent the VM from executing the literal data.
    #
    # Args:
    #   - name: the label name (new label name)
    #   - type: the data type
    #   - value: the literal value the label will point to (literal)
    #
    %% static_def name type value:

        %- after: &

        jmp =after

        @ {name}
        dd {type} {value}

        @=after

    %endmacro


    # Define static export data of the given type and value.
    # Take care of adding a constant jump instruction to prevent the VM from executing the literal data.
    #
    # Args:
    #   - name: the label name (new label name)
    #   - type: the data type
    #   - value: the literal value the label will point to (literal)
    #
    %% export_static_def name type value:

        %- after: &

        jmp =after

        @@ {name}
        dd {type} {value}

        @=after

    %endmacro

