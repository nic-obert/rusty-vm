#!/bin/bash

echo

echo "Rust:"
find . -type f -name "*.rs" ! -wholename "**/target/*" | xargs wc -l | sort -nr | head -n 1

echo $'\nVM:'
find ./vm -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nAssembler:'
find ./assembler -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nOxide:'
find ./oxide -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nAssembly:'
find . -name "*.asm" | xargs wc -l | sort -nr

