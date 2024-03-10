# **Rusty Virtual Machine**

- [**Rusty Virtual Machine**](#rusty-virtual-machine)
  - [Project structure](#project-structure)
  - [License](#license)

A simple virtual machine and related tools, all written in Rust.  
I created this project to learn more about virtual machines, compilers, assembly languages, and all things low level.
  
This is just an amateur project and should not be used in a production environment.
There are a few known vulnerabilities, plus it's not very efficient compared to production-grade virtual machines.

## Project structure

- The [`vm`](vm) directory contains the code for the virtual machine.
- The [`assembler`](assembler) directory contains the code for the assembler.
  - [`assembler/lib`](assembler/lib) contains shared assembly libraries to include in the assembly source code.
- The [`disassembler`](disassembler) directory contains the code for the disassembler. **(currently outdated)**
- The [`rust_vm_lib`](rust_vm_lib) directory contains the code for the shared library used across all Rust tools.
- The [`oxide`](oxide) directory contains an AOT compiler that compiles a custom language to the VM's bytecode.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
