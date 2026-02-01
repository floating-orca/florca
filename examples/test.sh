#!/bin/bash

set -Eeuo pipefail

DIR="$(dirname "$(realpath "$0")")"
cd "$DIR"

usage() {
  echo "Usage: $0 [--filter <FILTER>] [EXAMPLE]..."
  echo "  --filter <FILTER>  Only run tests that match the regular expression"
  echo "  EXAMPLE            Run tests for given example(s)"
  echo "  --help             Show this help message"
}

filter=()

while [[ $# -gt 0 ]]; do
  case $1 in
    --help)
      usage
      exit 0
      ;;
    --filter)
      if [[ -n "${2:-}" ]]; then
        filter=(--filter "$2")
        shift 2
      else
        echo "Error: --filter requires an argument" >&2
        exit 1
      fi
      ;;
    --*)
      echo "Unknown flag: $1" >&2
      usage
      exit 1
      ;;
    *)
      break
      ;;
  esac
done

examples=()
for example in "$@"; do
  examples+=("/examples/$example")
done
if [ ${#examples[@]} -eq 0 ]; then
  examples=("/examples")
fi

docker run --rm -t \
  -v "$(pwd)":/examples \
  florca-e2e bats -r "${filter[@]}" "${examples[@]}"
