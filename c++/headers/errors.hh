#pragma once

#include "utils.hh"
#include <iostream>


namespace error {

    // Runtime virtual machine status and error codes
    enum class ErrorCodes : Byte {

        NO_ERROR,

        END_OF_FILE,
        INVALID_INPUT,
        GENERIC_ERROR

    };


    [[ noreturn ]] void fileNotReadable(const char* fileName);   
    [[ noreturn ]] void fileEmpty(const char* fileName); 

}


std::ostream& operator<<(std::ostream& os, const error::ErrorCodes& error);

