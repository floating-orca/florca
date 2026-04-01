#!/bin/bash

set -Eeuo pipefail

DIR="$(dirname "$(realpath "$0")")"
cd "$DIR"

docker build \
	--build-arg CARGO_PROFILE="${CARGO_PROFILE:-dev}" \
	-f ./Dockerfile .. -t florca-e2e
