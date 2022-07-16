#include "processor.hh"
#include <stdexcept>
#include <iostream>


using namespace processor;


constexpr inline uint64* Processor::getRegister(Registers reg) {
    return &registers[static_cast<Byte>(reg)];
}


inline Byte* uint64ToBytes(uint64* value) {
    return (Byte*)value;
}

inline Byte* uint32ToBytes(uint32* value) {
    return (Byte*)value;
}

inline Byte* uint16ToBytes(uint16* value) {
    return (Byte*)value;
}


inline Byte* uint8ToBytes(uint8* value) {
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


constexpr inline uint64* bytesToUint64(Byte* bytes) {
    return (uint64*)bytes;
}

constexpr inline uint32* bytesToUint32(Byte* bytes) {
    return (uint32*)bytes;
}

constexpr inline uint16* bytesToUint16(Byte* bytes) {
    return (uint16*)bytes;
}

constexpr inline uint8* bytesToUint8(Byte* bytes) {
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
    const Registers reg = static_cast<Registers>(nextByteCode());
    (*getRegister(reg)) ++;
    setArithmeticalFlags(*getRegister(reg), 0);
}


void Processor::handle_inc_addr_in_reg() {
    const Byte size = nextByteCode();
    const Registers reg = static_cast<Registers>(nextByteCode());
    const Address address = *getRegister(reg);
    Byte* bytes = memory.getBytesMutable(address);
    
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
            throw std::runtime_error("Invalid size");
    }
}


void Processor::handle_inc_addr_literal() {
    const Byte size = nextByteCode();
    const Byte* addressBytes = nextByteCode(sizeof(Address));
    // TODO: implement

}

