#include "processor.hh"
#include "errors.hh"
#include <stdexcept>
#include <iostream>
#include <limits>


using namespace processor;
using namespace error;


constexpr inline Registers byteToRegister(Byte byte) {
    return static_cast<Registers>(byte);
}


constexpr inline uint64* Processor::getRegister(Registers reg) {
    return &registers[static_cast<Byte>(reg)];
}


inline Byte* uint64ToBytes(const uint64* value) {
    return (Byte*)value;
}

inline Byte* uint32ToBytes(const uint32* value) {
    return (Byte*)value;
}

inline Byte* uint16ToBytes(const uint16* value) {
    return (Byte*)value;
}


inline Byte* uint8ToBytes(const uint8* value) {
    return (Byte*)value;
}


inline uint64* bytesToUint64(const Byte* bytes) {
    return (uint64*)bytes;
}

inline uint32* bytesToUint32(const Byte* bytes) {
    return (uint32*)bytes;
}

inline uint16* bytesToUint16(const Byte* bytes) {
    return (uint16*)bytes;
}

inline uint8* bytesToUint8(const Byte* bytes) {
    return (uint8*)bytes;
}


Address Processor::addressFromByteCode() {
    return *bytesToUint64(nextByteCode(sizeof(Address)));
}


Processor::Processor(size_t stackSize, size_t videoSize)
: memory(stackSize, videoSize) 
{

}


Processor::~Processor() {

}


void Processor::clearVolatileRegisters() {
    *getRegister(Registers::EXIT) = 0;
}


void Processor::setArithmeticalFlags(int64 result, uint64 remainder) {
    *getRegister(Registers::ZERO_FLAG) = result == 0;
    *getRegister(Registers::SIGN_FLAG) = result < 0;
    *getRegister(Registers::REMAINDER_FLAG) = remainder;
}


void Processor::pushStackBytes(const Byte* bytes, size_t size) {
    memory.setBytes(
        *getRegister(Registers::STACK_POINTER),
        bytes,
        size
    );
    *getRegister(Registers::STACK_POINTER) += size;
}
            

void Processor::pushStack(uint64 value) {
    pushStackBytes((Byte*)&value, sizeof(value));
}


const Byte* Processor::popStackBytes(size_t size) {
    *getRegister(Registers::STACK_POINTER) -= size;
    return memory.getBytes(
        *getRegister(Registers::STACK_POINTER),
        size
    );
}


const Byte* Processor::nextByteCode(Byte size) {
    const uint64 pc = *getRegister(Registers::PROGRAM_COUNTER);
    *getRegister(Registers::PROGRAM_COUNTER) += size;
    return memory.getBytes(pc, size);
}


Byte Processor::nextByteCode() {
    const uint64 pc = *getRegister(Registers::PROGRAM_COUNTER);
    (*getRegister(Registers::PROGRAM_COUNTER)) ++;
    return memory.getByte(pc);
}


ErrorCodes Processor::execute(Byte* byteCode, size_t size, bool verbose) {
    // Load the byte code into memory
    pushStackBytes(byteCode, size);

    running = true;
    if (verbose) {
        runVerbose();
    } else {
        run();
    }

    // Return the error code of the program at exit
    return static_cast<ErrorCodes>(*getRegister(Registers::EXIT));
}


void Processor::run() {
    while (running) {

        Byte opCode = nextByteCode();

        handle_instruction(opCode);

        clearVolatileRegisters();
    }
}


void Processor::runVerbose() {
    while (running) {

        Byte opCode = nextByteCode();

        std::cout << "PC: " << *getRegister(Registers::PROGRAM_COUNTER) << ", "
            << "opcode: " << (ByteCodes)opCode << std::endl;

        handle_instruction(opCode);

        clearVolatileRegisters();
    }
}


inline void Processor::handle_instruction(Byte instruction) {
    switch (instruction) {
        case static_cast<int>(ByteCodes::ADD):
            handle_add();
            break;
        case static_cast<int>(ByteCodes::SUB):
            handle_sub();
            break;
        case static_cast<int>(ByteCodes::MUL):
            handle_mul();
            break;
        case static_cast<int>(ByteCodes::DIV):
            handle_div();
            break;
        case static_cast<int>(ByteCodes::MOD):
            handle_mod();
            break;
        case static_cast<int>(ByteCodes::INC_REG):
            handle_inc_reg();
            break;
        case static_cast<int>(ByteCodes::INC_ADDR_IN_REG):
            handle_inc_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::INC_ADDR_LITERAL):
            handle_inc_addr_literal();
            break;
        case static_cast<int>(ByteCodes::DEC_REG):
            handle_dec_reg();
            break;
        case static_cast<int>(ByteCodes::DEC_ADDR_IN_REG):
            handle_dec_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::DEC_ADDR_LITERAL):
            handle_dec_addr_literal();
            break;
        case static_cast<int>(ByteCodes::NO_OPERATION):
            handle_no_operation();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_REG_FROM_REG):
            handle_move_into_reg_from_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG):
            handle_move_into_reg_from_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_REG_FROM_CONST):
            handle_move_into_reg_from_const();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL):
            handle_move_into_reg_from_addr_literal();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG):   
            handle_move_into_addr_in_reg_from_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG):
            handle_move_into_addr_in_reg_from_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST):
            handle_move_into_addr_in_reg_from_const();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL):
            handle_move_into_addr_in_reg_from_addr_literal();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG):
            handle_move_into_addr_literal_from_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG):
            handle_move_into_addr_literal_from_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST):
            handle_move_into_addr_literal_from_const();
            break;
        case static_cast<int>(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL):
            handle_move_into_addr_literal_from_addr_literal();
            break;
        case static_cast<int>(ByteCodes::PUSH_FROM_REG):
            handle_push_from_reg();
            break;
        case static_cast<int>(ByteCodes::PUSH_FROM_ADDR_IN_REG):
            handle_push_from_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::PUSH_FROM_CONST):
            handle_push_from_const();
            break;
        case static_cast<int>(ByteCodes::PUSH_FROM_ADDR_LITERAL):
            handle_push_from_addr_literal();
            break;
        case static_cast<int>(ByteCodes::POP_INTO_REG):
            handle_pop_into_reg();
            break;
        case static_cast<int>(ByteCodes::POP_INTO_ADDR_IN_REG):
            handle_pop_into_addr_in_reg();
            break;
        case static_cast<int>(ByteCodes::POP_INTO_ADDR_LITERAL):
            handle_pop_into_addr_literal();
            break;
        case static_cast<int>(ByteCodes::JUMP):
            handle_jump();
            break;
        case static_cast<int>(ByteCodes::JUMP_IF_TRUE_REG):
            handle_jump_if_true_reg();
            break;
        case static_cast<int>(ByteCodes::JUMP_IF_FALSE_REG):
            handle_jump_if_false_reg();
            break;
        case static_cast<int>(ByteCodes::COMPARE_REG_REG):
            handle_compare_reg_reg();
            break;
        case static_cast<int>(ByteCodes::PRINT):
            handle_print();
            break;
        case static_cast<int>(ByteCodes::PRINT_STRING):
            handle_print_string();
            break;
        case static_cast<int>(ByteCodes::INPUT_INT):
            handle_input_int();
            break;
        case static_cast<int>(ByteCodes::INPUT_STRING):
            handle_input_string();
            break;
        case static_cast<int>(ByteCodes::EXIT):
            handle_exit();
            break;
    }
}


inline void Processor::handle_add() {
    *getRegister(Registers::A) += *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


inline void Processor::handle_sub() {
    *getRegister(Registers::A) -= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


inline void Processor::handle_mul() {
    *getRegister(Registers::A) *= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


inline void Processor::handle_div() {
    const uint64 remainder = *getRegister(Registers::A) % *getRegister(Registers::B);
    *getRegister(Registers::A) /= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), remainder);
}


inline void Processor::handle_mod() {
    *getRegister(Registers::A) %= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


inline void Processor::handle_inc_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    (*getRegister(destReg)) ++;
    setArithmeticalFlags(*getRegister(destReg), 0);
}


void Processor::incrementUnsigned(Byte* bytes, Byte size) {
    switch (size) {
        case 1:
            *bytesToUint8(bytes) += 1;
            setArithmeticalFlags(*bytesToUint8(bytes), 0);
            break;
        case 2:
            *bytesToUint16(bytes) += 1;
            setArithmeticalFlags(*bytesToUint16(bytes), 0);
            break;
        case 4:
            *bytesToUint32(bytes) += 1;
            setArithmeticalFlags(*bytesToUint32(bytes), 0);
            break;
        case 8:
            *bytesToUint64(bytes) += 1;
            setArithmeticalFlags(*bytesToUint64(bytes), 0);
            break;
        default:
            throw std::runtime_error("Invalid size: " + std::to_string(size));
    }
}


void Processor::decrementUnsigned(Byte* bytes, Byte size) {
    switch (size) {
        case 1:
            *bytesToUint8(bytes) -= 1;
            setArithmeticalFlags(*bytesToUint8(bytes), 0);
            break;
        case 2:
            *bytesToUint16(bytes) -= 1;
            setArithmeticalFlags(*bytesToUint16(bytes), 0);
            break;
        case 4:
            *bytesToUint32(bytes) -= 1;
            setArithmeticalFlags(*bytesToUint32(bytes), 0);
            break;
        case 8:
            *bytesToUint64(bytes) -= 1;
            setArithmeticalFlags(*bytesToUint64(bytes), 0);
            break;
        default:
            throw std::runtime_error("Invalid size: " + std::to_string(size));
    }
}


inline void Processor::handle_inc_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address address = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(address);
    
    incrementUnsigned(bytes, size);
}


inline void Processor::handle_inc_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    Byte* bytes = memory.getBytesMutable(destAddress);

    incrementUnsigned(bytes, size);
}


inline void Processor::handle_dec_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    (*getRegister(destReg)) --;
    setArithmeticalFlags(*getRegister(destReg), 0);
}


inline void Processor::handle_dec_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(destAddress);
    
    decrementUnsigned(bytes, size);
}


inline void Processor::handle_dec_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    Byte* bytes = memory.getBytesMutable(destAddress);

    decrementUnsigned(bytes, size);
}


inline void Processor::handle_no_operation() {
    // Do nothing
}


inline void Processor::handle_move_into_reg_from_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    const Registers srcReg = byteToRegister(nextByteCode());
    *getRegister(destReg) = *getRegister(srcReg);
}


void Processor::moveBytesIntoRegister(const Byte* bytes, Byte size, Registers destReg) {
    switch (size) {
        case 1:
            *getRegister(destReg) = *bytesToUint8(bytes);
            break;
        case 2:
            *getRegister(destReg) = *bytesToUint16(bytes);
            break;
        case 4:
            *getRegister(destReg) = *bytesToUint32(bytes);
            break;
        case 8:
            *getRegister(destReg) = *bytesToUint64(bytes);
            break;
        default:
            throw std::runtime_error("Invalid size: " + std::to_string(size));
    }
}


inline void Processor::handle_move_into_reg_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers destReg = byteToRegister(nextByteCode());
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(srcAddress);

    moveBytesIntoRegister(bytes, size, destReg);
}


inline void Processor::handle_move_into_reg_from_const() {
    const Byte size = nextByteCode();
    const Registers destReg = byteToRegister(nextByteCode());
    const Byte* bytes = nextByteCode(size);
    
    moveBytesIntoRegister(bytes, size, destReg);
}


inline void Processor::handle_move_into_reg_from_addr_literal() {
    const Byte size = nextByteCode();
    const Registers destReg = byteToRegister(nextByteCode());
    const Address srcAddress = addressFromByteCode();
    Byte* bytes = memory.getBytesMutable(srcAddress);

    moveBytesIntoRegister(bytes, size, destReg);
}


void Processor::moveRegisterIntoAddress(const Registers srcReg, const Address destAddress, const Byte size) {
    Byte* bytes = memory.getBytesMutable(destAddress);

    switch (size) {
        case 1:
            *bytesToUint8(bytes) = *getRegister(srcReg);
            break;
        case 2:
            *bytesToUint16(bytes) = *getRegister(srcReg);
            break;
        case 4:
            *bytesToUint32(bytes) = *getRegister(srcReg);
            break;
        case 8:
            *bytesToUint64(bytes) = *getRegister(srcReg);
            break;
        default:
            throw std::runtime_error("Invalid size: " + std::to_string(size));
    }
}


inline void Processor::handle_move_into_addr_in_reg_from_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Registers srcReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);

    moveRegisterIntoAddress(srcReg, destAddress, size);           
}


inline void Processor::handle_move_into_addr_in_reg_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers destAddressReg = byteToRegister(nextByteCode());
    const Registers srcAddressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(destAddressReg);
    const Address srcAddress = *getRegister(srcAddressReg);
    
    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_move_into_addr_in_reg_from_const() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);
    const Byte* bytes = nextByteCode(size);
    
    memory.setBytes(destAddress, bytes, size);
}


inline void Processor::handle_move_into_addr_in_reg_from_addr_literal() {
    const Byte size = nextByteCode();
    const Registers reg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(reg);
    const Address srcAddress = addressFromByteCode();

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_move_into_addr_literal_from_reg() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Registers srcReg = byteToRegister(nextByteCode());

    memory.setBytes(destAddress, uint64ToBytes(getRegister(srcReg)), size);
}


inline void Processor::handle_move_into_addr_literal_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_move_into_addr_literal_from_const() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Byte* bytes = nextByteCode(size);
    
    memory.setBytes(destAddress, bytes, size);
}


inline void Processor::handle_move_into_addr_literal_from_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Address srcAddress = addressFromByteCode();

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_push_from_reg() {
    const Registers srcReg = byteToRegister(nextByteCode());
    pushStack(*getRegister(srcReg));
}


inline void Processor::handle_push_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);

    pushStackBytes(memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_push_from_const() {
    const Byte size = nextByteCode();
    const Byte* bytes = nextByteCode(size);

    pushStackBytes(bytes, size);
}


inline void Processor::handle_push_from_addr_literal() {
    const Byte size = nextByteCode();
    const Address srcAddress = addressFromByteCode();

    pushStackBytes(memory.getBytes(srcAddress, size), size);
}


inline void Processor::handle_pop_into_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    const Byte* bytes = popStackBytes(sizeof(uint64));
    *getRegister(destReg) = *bytesToUint64(bytes);
}


inline void Processor::handle_pop_into_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);

    memory.setBytes(destAddress, popStackBytes(size), size);
}


inline void Processor::handle_pop_into_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();

    memory.setBytes(destAddress, popStackBytes(size), size);
}


inline void Processor::handle_jump() {
    *getRegister(Registers::PROGRAM_COUNTER) = addressFromByteCode();
}


inline void Processor::handle_jump_if_true_reg() {
    const Address target = addressFromByteCode();
    const Registers testReg = byteToRegister(nextByteCode());

    if (*getRegister(testReg)) {
        *getRegister(Registers::PROGRAM_COUNTER) = target;
    }
}


inline void Processor::handle_jump_if_false_reg() {
    const Address target = addressFromByteCode();
    const Registers testReg = byteToRegister(nextByteCode());

    if (!*getRegister(testReg)) {
        *getRegister(Registers::PROGRAM_COUNTER) = target;
    }
}


inline void Processor::handle_compare_reg_reg() {
    const Registers reg1 = byteToRegister(nextByteCode());
    const Registers reg2 = byteToRegister(nextByteCode());

    setArithmeticalFlags(*getRegister(reg1) - *getRegister(reg2), 0);
}


inline void Processor::handle_compare_reg_const() {
    const Byte size = nextByteCode();
    const Registers reg = byteToRegister(nextByteCode());
    const uint64 value = *bytesToUint64(nextByteCode(size));

    setArithmeticalFlags(*getRegister(reg) - value, 0);
}


inline void Processor::handle_compare_const_reg() {
    const Byte size = nextByteCode();
    const uint64 value = *bytesToUint64(nextByteCode(size));
    const Registers reg = byteToRegister(nextByteCode());
    
    setArithmeticalFlags(value - *getRegister(reg), 0);
}


inline void Processor::handle_compare_const_const() {
    const Byte size = nextByteCode();
    const uint64 value1 = *bytesToUint64(nextByteCode(size));
    const uint64 value2 = *bytesToUint64(nextByteCode(size));
    
    setArithmeticalFlags(value1 - value2, 0);
}


inline void Processor::handle_print() {
    const uint64 value = *getRegister(Registers::PRINT);
    std::cout << value;
    std::flush(std::cout);
}


inline void Processor::handle_print_string() {
    Address srcAddress = *getRegister(Registers::PRINT);
    for (
        Byte byte = memory.getByte(srcAddress);
        byte != 0;
        byte = memory.getByte(++srcAddress)
    ) {
        std::cout << (char) byte;
    }
    std::flush(std::cout);
}


inline void Processor::handle_input_int() {
    std::cin >> *getRegister(Registers::INPUT);

    if (std::cin.eof()) {
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::END_OF_FILE);
        return;
    }
    
    if (std::cin.fail()) {
        std::cin.clear();
        std::cin.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::INVALID_INPUT);
        return;
    }

    if (std::cin.bad()) {
        std::cin.clear();
        std::cin.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::GENERIC_ERROR);
        return;
    }

    *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::NO_ERROR);
}


inline void Processor::handle_input_string() {
    std::string input;
    std::getline(std::cin, input);

    if (std::cin.eof()) {
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::END_OF_FILE);
        return;
    }

    if (std::cin.fail()) {
        std::cin.clear();
        std::cin.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::INVALID_INPUT);
        return;
    }

    if (std::cin.bad()) {
        std::cin.clear();
        std::cin.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::GENERIC_ERROR);
        return;
    }

    *getRegister(Registers::ERROR) = static_cast<uint64>(ErrorCodes::NO_ERROR);
    *getRegister(Registers::INPUT) = static_cast<uint64>(input.size());
    pushStackBytes((const Byte*) input.c_str(), input.size());
}


inline void Processor::handle_exit() {
    running = false;
}

