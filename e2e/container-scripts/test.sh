#!/bin/bash

set -Eeuo pipefail

florca deploy -w examples/siblings

florca run -d siblings --wait | tee /tmp/florca-output.log
output=$(cat /tmp/florca-output.log)
if [[ $output == *"Output: [10,9,8,7,6]"* ]]; then
    echo "Test passed"
else
    echo "Test failed"
    exit 1
fi
