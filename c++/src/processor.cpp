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


void Processor::pushStackBytes(Byte* bytes, size_t size) {
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


Byte* Processor::popStackBytes(size_t size) {
    *getRegister(Registers::STACK_POINTER) -= size;
    return memory.getBytes(
        *getRegister(Registers::STACK_POINTER),
        size
    );
}


const Byte* Processor::nextByteCode(Byte size) {
    uint64 pc = *getRegister(Registers::PROGRAM_COUNTER);
    *getRegister(Registers::PROGRAM_COUNTER) += size;
    return memory.getBytes(pc, size);
}


void Processor::execute(Byte* byteCode, size_t size) {
    // Load the byte code into memory
    pushStackBytes(byteCode, size);

    running = true;
    while (running) {

        Byte opCode = nextByteCode(1)[0];

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
    uint64 remainder = *getRegister(Registers::A) % *getRegister(Registers::B);
    *getRegister(Registers::A) /= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), remainder);
}


void Processor::handle_mod() {
    *getRegister(Registers::A) %= *getRegister(Registers::B);
    setArithmeticalFlags(*getRegister(Registers::A), 0);
}

