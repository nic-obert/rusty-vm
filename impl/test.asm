
.include:

    "stdio/print.asm"
    "archlib.asm"


.data:

    @MSG
    db [107 101 97 48 10 0]

    @TEST_NUM
    dn 8 80


.text:

@start


    !print_str MSG


