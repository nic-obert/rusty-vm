#include "utils.hh"
#include <fstream>


size_t loadFileBytes(const char* path, Byte* content) {
    
    std::ifstream file(path);

    if (file.bad()) {
        return 0;
    }

    // Get the size of the file
    file.seekg(0, file.end);
    size_t size = file.tellg();
    file.seekg(0, file.beg);

    // Load the file into memory
    file.read(reinterpret_cast<char*>(content), size);
    file.close();

    return size;
}

