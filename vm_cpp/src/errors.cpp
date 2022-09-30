#include "errors.hh"


[[ noreturn ]] void error::fileNotReadable(const char* fileName) {
    std::cerr << "File " << fileName << " is not readable (not found or bad)" << std::endl;
    exit(1);
}


[[ noreturn ]] void error::fileEmpty(const char* fileName) {
    std::cerr << "File " << fileName << " is empty" << std::endl;
    exit(1);
}


std::ostream& operator<<(std::ostream& os, const error::ErrorCodes& error) {
    switch (error) {
        case error::ErrorCodes::END_OF_FILE:
            os << "END_OF_FILE";
            break;
        case error::ErrorCodes::INVALID_INPUT:
            os << "INVALID_INPUT";
            break;
        case error::ErrorCodes::GENERIC_ERROR:
            os << "GENERIC_ERROR";
            break;
        default:
            os << "Unknown error code: " << static_cast<Byte>(error);
            break;
    }
    return os;
}

