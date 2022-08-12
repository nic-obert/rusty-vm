#include "processor.hh"
#include "errors.hh"
#include <stdexcept>
#include <iostream>
#include <limits>


using namespace processor;
using namespace error;


Address Processor::addressFromByteCode() {
    return *bytesToUint64(nextByteCode(sizeof(Address)));
}


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


typedef Byte* (*UintToBytes)(void* value);
static constexpr const UintToBytes UINT_TO_BYTES_TABLE[] = {
    nullptr,                        // 0
    (UintToBytes) uint8ToBytes,     // 1
    (UintToBytes) uint16ToBytes,    // 2
    nullptr,                        // 3
    (UintToBytes) uint32ToBytes,    // 4
    nullptr,                        // 5
    nullptr,                        // 6
    nullptr,                        // 7
    (UintToBytes) uint64ToBytes,    // 8
};


constexpr inline uint64* bytesToUint64(const Byte* bytes) {
    return (uint64*)bytes;
}

constexpr inline uint32* bytesToUint32(const Byte* bytes) {
    return (uint32*)bytes;
}

constexpr inline uint16* bytesToUint16(const Byte* bytes) {
    return (uint16*)bytes;
}

constexpr inline uint8* bytesToUint8(const Byte* bytes) {
    return (uint8*)bytes;
}


Processor::Processor(size_t memorySize) :
    memory(memorySize)
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


void Processor::execute(Byte* byteCode, size_t size, bool verbose) {
    // Load the byte code into memory
    pushStackBytes(byteCode, size);

    running = true;
    if (verbose) {
        runVerbose();
    } else {
        run();
    }

    // TODO: implement exiting from the program
}


void Processor::run() {
    while (running) {

        Byte opCode = nextByteCode();

        (this->*INSTRUCTION_HANDLERS[opCode])();

        clearVolatileRegisters();
    }
}


void Processor::runVerbose() {
    while (running) {

        Byte opCode = nextByteCode();

        std::cout << "PC: " << *getRegister(Registers::PROGRAM_COUNTER) << ", "
            << "opcode: " << (ByteCodes)opCode << std::endl;

        (this->*INSTRUCTION_HANDLERS[opCode])();

        clearVolatileRegisters();
    }
}


void Processor::handle_add() {
    *getRegister(Registers::A) += *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


void Processor::handle_sub() {
    *getRegister(Registers::A) -= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


void Processor::handle_mul() {
    *getRegister(Registers::A) *= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


void Processor::handle_div() {
    const uint64 remainder = *getRegister(Registers::A) % *getRegister(Registers::B);
    *getRegister(Registers::A) /= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), remainder);
}


void Processor::handle_mod() {
    *getRegister(Registers::A) %= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}


void Processor::handle_inc_reg() {
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


void Processor::handle_inc_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address address = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(address);
    
    incrementUnsigned(bytes, size);
}


void Processor::handle_inc_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    Byte* bytes = memory.getBytesMutable(destAddress);

    incrementUnsigned(bytes, size);
}


void Processor::handle_dec_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    (*getRegister(destReg)) --;
    setArithmeticalFlags(*getRegister(destReg), 0);
}


void Processor::handle_dec_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(destAddress);
    
    decrementUnsigned(bytes, size);
}


void Processor::handle_dec_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    Byte* bytes = memory.getBytesMutable(destAddress);

    decrementUnsigned(bytes, size);
}


void Processor::handle_no_operation() {
    // Do nothing
}


void Processor::handle_move_into_reg_from_reg() {
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


void Processor::handle_move_into_reg_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers destReg = byteToRegister(nextByteCode());
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);
    Byte* bytes = memory.getBytesMutable(srcAddress);

    moveBytesIntoRegister(bytes, size, destReg);
}


void Processor::handle_move_into_reg_from_const() {
    const Byte size = nextByteCode();
    const Registers destReg = byteToRegister(nextByteCode());
    const Byte* bytes = nextByteCode(size);
    
    moveBytesIntoRegister(bytes, size, destReg);
}


void Processor::handle_move_into_reg_from_addr_literal() {
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


void Processor::handle_move_into_addr_in_reg_from_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Registers srcReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);

    moveRegisterIntoAddress(srcReg, destAddress, size);           
}


void Processor::handle_move_into_addr_in_reg_from_const() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);
    const Byte* bytes = nextByteCode(size);
    
    memory.setBytes(destAddress, bytes, size);
}


void Processor::handle_move_into_addr_in_reg_from_addr_literal() {
    const Byte size = nextByteCode();
    const Registers reg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(reg);
    const Address srcAddress = addressFromByteCode();

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


void Processor::handle_move_into_addr_literal_from_reg() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Registers srcReg = byteToRegister(nextByteCode());

    memory.setBytes(destAddress, uint64ToBytes(getRegister(srcReg)), size);
}


void Processor::handle_move_into_addr_literal_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


void Processor::handle_move_into_addr_literal_from_const() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Byte* bytes = nextByteCode(size);
    
    memory.setBytes(destAddress, bytes, size);
}


void Processor::handle_move_into_addr_literal_from_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();
    const Address srcAddress = addressFromByteCode();

    memory.setBytes(destAddress, memory.getBytes(srcAddress, size), size);
}


void Processor::handle_push_from_reg() {
    const Registers srcReg = byteToRegister(nextByteCode());
    pushStack(*getRegister(srcReg));
}


void Processor::handle_push_from_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address srcAddress = *getRegister(addressReg);

    pushStackBytes(memory.getBytes(srcAddress, size), size);
}


void Processor::handle_push_from_const() {
    const Byte size = nextByteCode();
    const Byte* bytes = nextByteCode(size);

    pushStackBytes(bytes, size);
}


void Processor::handle_push_from_addr_literal() {
    const Byte size = nextByteCode();
    const Address srcAddress = addressFromByteCode();

    pushStackBytes(memory.getBytes(srcAddress, size), size);
}


void Processor::handle_pop_into_reg() {
    const Registers destReg = byteToRegister(nextByteCode());
    const Byte* bytes = popStackBytes(sizeof(uint64));
    *getRegister(destReg) = *bytesToUint64(bytes);
}


void Processor::handle_pop_into_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers addressReg = byteToRegister(nextByteCode());
    const Address destAddress = *getRegister(addressReg);

    memory.setBytes(destAddress, popStackBytes(size), size);
}


void Processor::handle_pop_into_addr_literal() {
    const Byte size = nextByteCode();
    const Address destAddress = addressFromByteCode();

    memory.setBytes(destAddress, popStackBytes(size), size);
}


void Processor::handle_jump() {
    *getRegister(Registers::PROGRAM_COUNTER) = addressFromByteCode();
}


void Processor::handle_jump_if_true_reg() {
    const Address target = addressFromByteCode();
    const Registers testReg = byteToRegister(nextByteCode());

    if (*getRegister(testReg)) {
        *getRegister(Registers::PROGRAM_COUNTER) = target;
    }
}


void Processor::handle_jump_if_false_reg() {
    const Address target = addressFromByteCode();
    const Registers testReg = byteToRegister(nextByteCode());

    if (!*getRegister(testReg)) {
        *getRegister(Registers::PROGRAM_COUNTER) = target;
    }
}


void Processor::handle_compare_reg_reg() {
    const Registers reg1 = byteToRegister(nextByteCode());
    const Registers reg2 = byteToRegister(nextByteCode());

    setArithmeticalFlags(*getRegister(reg1) - *getRegister(reg2), 0);
}


void Processor::handle_compare_reg_const() {
    const Byte size = nextByteCode();
    const Registers reg = byteToRegister(nextByteCode());
    const uint64 value = *bytesToUint64(nextByteCode(size));

    setArithmeticalFlags(*getRegister(reg) - value, 0);
}


void Processor::handle_compare_const_reg() {
    const Byte size = nextByteCode();
    const uint64 value = *bytesToUint64(nextByteCode(size));
    const Registers reg = byteToRegister(nextByteCode());
    
    setArithmeticalFlags(value - *getRegister(reg), 0);
}


void Processor::handle_compare_const_const() {
    const Byte size = nextByteCode();
    const uint64 value1 = *bytesToUint64(nextByteCode(size));
    const uint64 value2 = *bytesToUint64(nextByteCode(size));
    
    setArithmeticalFlags(value1 - value2, 0);
}


void Processor::handle_print() {
    const uint64 value = *getRegister(Registers::PRINT);
    std::cout << value;
    std::flush(std::cout);
}


void Processor::handle_print_string() {
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


void Processor::handle_input_int() {
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


void Processor::handle_input_string() {
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


void Processor::handle_exit() {
    running = false;
}

