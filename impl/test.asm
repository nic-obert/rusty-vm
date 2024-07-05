
.include:

    "stdio.asm"

.data:

    %- foo: &

    @=foo
    ds "Hello\0"

    @test
    ds "test\0"

.text:
    
    !println_str test
    !println_str =foo
