#!/bin/bash

set -Eeuo pipefail

DIR="$(dirname "$(realpath "$0")")"
cd "$DIR"

docker build -f ./Dockerfile .. -t florca-e2e
