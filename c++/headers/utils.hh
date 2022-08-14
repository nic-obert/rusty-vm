#pragma once


typedef unsigned char Byte;

typedef unsigned long size_t;

typedef unsigned long long uint64;
typedef long long int64;
typedef unsigned int uint32;
typedef int int32;
typedef unsigned short uint16;
typedef short int16;
typedef unsigned char uint8;
typedef char int8;


/*
    Try to load a file into memory.
    Returns the size of the file in bytes if successful.
    Returns 0 if the file could not be loaded or is empty.
*/
size_t loadFileBytes(const char* path, Byte* content);

