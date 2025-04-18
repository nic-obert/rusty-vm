# Fixed-size object allocator
#
# Heap structure:
# 0x0000 ------------------------------------------- 0xFFFF
# [ Heap metadata ] [ ------ Cells ------> ] [ <--- Stack ]
#
# Heap metadata structure:
# [ Object size (8 bytes) ] [ heap end ptr (8 bytes) ]
#
# Cell structure:
# [ Free (1 byte) ] [ Cell data ]
#
# Warning: each the allocator instance will have a static lifetime
#

.include:

    @@ "init.asm"
    @@ "alloc.asm"
    @@ "free.asm"
