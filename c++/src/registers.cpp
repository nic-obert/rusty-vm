#include "registers.hh"

#include <stdexcept>
#include <unordered_map>


using namespace registers;


static constexpr const char* const REGISTER_NAMES[] = {
    "A",
    "B",
    
    "C",
    "D",
    
    "EXIT",
    "INPUT",
    "ERROR",
    "PRINT",
    
    "STACK_POINTER",
    "PROGRAM_COUNTER",
    
    "ZERO_FLAG",
    "SIGN_FLAG",
    "REMAINDER_FLAG"
};


constexpr inline const char* registers::getRegisterName(Registers reg) {
    return REGISTER_NAMES[static_cast<Byte>(reg)];
}

static const std::unordered_map<const char*, const Registers> const REGISTERS_TABLE = {
    {"A", Registers::A},
    {"B", Registers::B},
    {"C", Registers::C},
    {"D", Registers::D},
    {"EXIT", Registers::EXIT},
    {"INPUT", Registers::INPUT},
    {"ERROR", Registers::ERROR},
    {"PRINT", Registers::PRINT},
    {"STACK_POINTER", Registers::STACK_POINTER},
    {"PROGRAM_COUNTER", Registers::PROGRAM_COUNTER},
    {"ZERO_FLAG", Registers::ZERO_FLAG},
    {"SIGN_FLAG", Registers::SIGN_FLAG},
    {"REMAINDER_FLAG", Registers::REMAINDER_FLAG}
};


constexpr inline Registers registers::getRegisterByName(const char* name) {
    return REGISTERS_TABLE.at(name);
}

