#!/bin/bash

echo

echo "Rust:"
find . -type f -name "*.rs" ! -wholename "**/target/*" | xargs wc -l | tail -n 1

echo $'\nVM:'
find ./vm -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nAssembler:'
find ./assembler -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nOxide:'
find ./oxide -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nDebugger:'
find ./debugger -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\ndebug info viewer:'
find ./debug_info_viewer -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nShared lib:'
find ./rusty_vm_lib -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nAssembly:'
find . -name "*.asm" | xargs wc -l | sort -nr

echo
