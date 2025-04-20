# Btree global allocator
#
# Components:
# - free table (binary tree)
# - pocket object allocator for the free table
# - actual heap buffer
#
# Node structure:
# - Type: [parent/free leaf/occupied leaf] (1 byte)
# - Left child: non-null ptr
# - Right child: non-null ptr
#
# Warning: the allocator instance has a static lifetime
# Warning: a single btree allocator instance can be created
#

.include:

    @@ "init.asm"
    @@ "alloc.asm"
    @@ "free.asm"
    @@ "info.asm"
