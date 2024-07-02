# array
# Fixed size array allocated on the heap

# Memory structure:
#   - item_size: 8 bytes
#   - length: 8 bytes
#   - data: (item_size * length) bytes


.include:

    @@ "array/get.asm"
    @@ "array/item_size.asm"
    @@ "array/length.asm"
    @@ "array/new.asm"
    @@ "array/set.asm"
    @@ "array/resize.asm"
    @@ "array/shared.asm"

