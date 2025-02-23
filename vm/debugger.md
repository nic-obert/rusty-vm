# Debugger devnotes

[debugger UI draft](https://www.canva.com/design/DAGfBGWyinA/IgXClCdMUAQImRa1Tjb_ZA/edit)

A debugging data format is required to allow the debugger to work properly, similar to the DWARF debugging format.
The VM already sets the pc to the address specified by the last 8 bytes of the binary, so there's no need to jump over the debug data section.

## Debugging features needed:
- Map machine instruction to source code (assembly) line and file
- Inspect memory arbitrarily and copy
- inspect registers
- inspect instructions before execution (get arguments)
- breakpoints (set in-place and at arbitrary instruction address)
- inspect debug symbols (labels, etc.) (separate window?)
- stop execution arbitrarily without a breakpoint
- advance execution by one instruction without setting a breakpoint manually
- continue execution while retaining the previous breakpoint, if present
- perform a core dump
- view the history of registers up to a limit (separate window?) (implemented in the debugger UI)
- close everything, stop debugging and terminate both the debugger and the vm processes

---

## VM-debugger interoperability
The debugger UI program shall connect to the VM process and work together via IPC.
The VM process shall launch the debugger UI process when ran with an apposite debug flag or mode (passing `-md` as in "mode debug" to the vm).
The VM process shall initiate IPC using a custom protocol, implemented in a static shared library that can be included and used by any debugger UI.

IPC protocol features:
- bind exactly two processes
- send messages bidirectionally
- non-blocking waiting for messages (the VM may poll for messages at the beginning of every cycle. Performance is not a priority in debug mode)

IPC requirements:
- advance by one instruction
- stop execution (and get register status)
- continue execution
- shared memory

### Stop/Continue execution
Possible implementations:
- set a byte in shared memory that signals the VM to stop or continue. (Would need to get registers state in some other way)
- use an IPC message to stop or resume the VM (and get the registers state on stop)

### Advance by one instruction
 0. Execution is stopped and current registers are available.
 1. Set a breakpoint on the next instruction.
 DOESN'T WORK IF THE CURRENT PC IS ON A JUMP INSTRUCTION. A branch interpreter may be used in this case to identify the correct instruction to set a breakpoint on.
 Also, check for memory bounds. The last instruction in the executable may also be the last byte in memory. The `exit` instruction may be treated as a jump instruction since it alters the program flow, and no breakpoint shall be placed after it.
 2. If the current pc has a breakpoint, restore the overwritten instruction at the current pc.
 3. Continue execution. The VM will stop at the next instruction because of the breakpoint.
 4. If the previous instruction had a breakpoint, restore the breakpoint.

### Core dumps
In the debugger UI, use a file picker to choose the path to save the core file to.
Core dumps shall not be implemented solely by the debugger UI process since, for a core dump to be useful, it must be internally consistent. While the dump is being performed, the VM shall stop execution momentarily to prevent the current VM state from changing during the dump.

A core dump feature may be implemented as a two-step process:
 1. Stop VM execution via a stop message. The VM shall respond to the stop message with the current register state.
 2. The debugger process then collects the relevant data from shared memory and, along with the register state, produces a core dump file.

### Memory operations and breakpoints
Memory manipulation may be implemented via shared memory and the memory may be directly edited by the debugger UI process.
This would also improve performance when reading memory from the VM process. Linux shared memory features may be used to implement this.
Raw memory sharing also allows the debugger UI process to read the debug information directly from the executable file.

breakpoint operations are implemented via direct memory editing.
The debugger UI is responsible for storing breakpoint information. Breakpoints may be implemented as a "set this specific memory address to this value", where the value is a breakpoint instruction.
The debugger UI stores where breakpoints are placed and what value was stored previous to the breakpoint instruction.
The breakpoint disabling feature is implemented by the debugger UI process via setting/unsetting the breakpoint instruction.


## The debug breakpoint instruction
This instruction shall be part of the VM instruction set.
When executed, the breakpoint instruction shall stop execution at the next cycle and notify the debugger.
The VM process may also communicate the current state of CPU registers to the debugger UI process.

## Shared memory
Shared memory shall be implemented via the operating system.
The VM process, when launched in debug mode, creates a memory mapping that will be shared with the debugger process.
The shared memory shall be organized this way:

LOW ---> HIGH
[CPU registers, manual update on VM stop] [Running 1 byte] [Terminate command 1 byte] [VM updated counter 1 byte] [VM memory, mapped via the OS]

- The running flag tells the VM to stop or continue execution.
- The terminate command tells the VM process to terminate itself.
- The VM updated counter is used to signal that an update was performed by the VM process.
For instance, if the debugger stops VM execution and wants to read the registers state, it shall wait until the VM updated counter is incremented.
The updated counter shall be the last field to be updated when changes to shared memory are made by the VM process.
This counter is necessary so that the debugger process knows whether the VM process has received the message and when it has finished responding.

The CPU registers region is placed first to keep it aligned to the size of a register entry (8 bytes).
