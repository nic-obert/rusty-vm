#!/bin/bash

echo "Rust:"
find . -type f -name "*.rs" ! -wholename "**/target/*" | xargs wc -l | sort -nr

echo $'\nAssembly:'
find . -name "*.asm" | xargs wc -l | sort -nr

