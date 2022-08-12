#pragma once

#include "utils.hh"


namespace error {

    enum class ErrorCodes : Byte {

        NO_ERROR,

        END_OF_FILE,
        INVALID_INPUT,
        GENERIC_ERROR

    };

}

