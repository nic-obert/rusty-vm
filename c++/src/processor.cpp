#include "processor.hh"


using namespace processor;


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


void Processor::execute(Byte* byteCode, size_t size) {
    // Load the byte code into memory
    pushStackBytes(byteCode, size);

    running = true;
    while (running) {

        Byte opCode = nextByteCode();

        (this->*INSTRUCTION_HANDLERS[opCode])();

        clearVolatileRegisters();

    }

    // TODO: implement exiting from the program
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
    const Byte* bytes = memory.getBytes(address, size);
    // TODO: to check if works (also endianness)
    // Convert the bytes to a uint64
    uint64 value = 0;
    for (int i = 0; i < size; i++) {
        value += bytes[i] << (8 * i);
    }
    value ++;
    // Convert the uint64 to bytes
    const Byte* newBytes = (Byte*)&value;
    // Write the new bytes to memory
    memory.setBytes(address, newBytes, size);
    // TODO: check if works (also endianness)

    setArithmeticalFlags(value, 0);
}


void Processor::handle_inc_addr_literal() {
    const Byte size = nextByteCode();
    const Byte* addressBytes = nextByteCode(sizeof(Address));
    // TODO: to check if works (also endianness)
    const Address address = *(Address*)addressBytes;
    const Byte* bytes = memory.getBytes(address, size);

    // Convert the bytes to a uint64

}

