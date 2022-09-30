#include "utils.hh"
#include "errors.hh"
#include <fstream>


size_t loadFileBytes(const char* path, Byte** content) {
    
    std::ifstream file(path);

    if (file.bad()) {
        error::fileNotReadable(path);
    }

    // Get the size of the file
    file.seekg(0, file.end);
    size_t size = file.tellg();

    if (size == 0) {
        error::fileEmpty(path);
    }

    file.seekg(0, file.beg);

    // Load the file into memory
    file.read(reinterpret_cast<char*>(content), size);
    file.close();

    return size;
}

