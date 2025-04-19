#!/bin/bash
# Remove all .out files in the current directory and subdirectories.

find . -name "*.out" -exec rm {} \;
