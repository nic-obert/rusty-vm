#!/bin/bash
# Remove all .bc files in the current directory and subdirectories.

find . -name "*.out" -exec rm {} \;
