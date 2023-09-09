# host_fs
# Access the host file system


.include:

    interrupts.asm


.text:

    # Host fs operation codes

    %- HOST_FS_EXISTS: 0
    %- HOST_FS_READ_ALL: 1
    %- HOST_FS_WRITE_ALL: 2
    %- HOST_FS_CREATE_FILE: 3
    %- HOST_FS_CREATE_DIR: 4

    # Constants

    %- HOST_FS_CODE_REG: print


    %% host_fs_exists file_path:

        mov8 r1 {file_path}
        mov1 =HOST_FS_CODE_REG =HOST_FS_EXISTS
        intr =HOST_FS_INTR
    
    %endmacro


    %% host_fs_read_all:

        mov1 =HOST_FS_CODE_REG =HOST_FS_READ_ALL
        intr =HOST_FS_INTR
    
    %endmacro


    %% host_fs_write_all:

        mov1 =HOST_FS_CODE_REG =HOST_FS_WRITE_ALL
        intr =HOST_FS_INTR
    
    %endmacro


    %% host_fs_create_file file_path:

        mov8 r1 {file_path}
        mov1 =HOST_FS_CODE_REG =HOST_FS_CREATE_FILE
        intr =HOST_FS_CODE_REG
    
    %endmacro


    %% host_fs_create_dir dir_path:

        mov8 r1 {dir_path}
        mov1 =HOST_FS_CODE_REG =HOST_FS_CREATE_DIR
        intr =HOST_FS_CODE_REG

    %endmacro

