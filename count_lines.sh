#!/bin/bash
find . -wholename "**/src/*" -type f | xargs wc -l | sort -nr
