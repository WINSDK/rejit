#!/bin/bash
set -e

cargo r '(ab)*c'
gcc example.c -L. -l matcher -o example
echo "Binary linked and written to 'example'."
./example "abc"
rm example libmatcher.so
