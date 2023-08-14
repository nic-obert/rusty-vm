#!/bin/bash

echo "Rust:"
find . -name "*.rs" | xargs wc -l | sort -nr

echo $'\nAssembly:'
find . -name "*.asm" | xargs wc -l | sort -nr

