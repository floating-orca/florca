#!/bin/bash

set -Eeuo pipefail

docker build -f e2e/Dockerfile . -t florca-e2e
docker run --rm florca-e2e
