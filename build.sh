#!/usr/bin/bash

set -x

cargo build --release

# ls target/release/*.exe
cp target/release/*.exe /c/Software/bin/

echo "DONE BUILD"

