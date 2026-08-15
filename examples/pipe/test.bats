#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow with stdin" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  # cat input.json | florca run ...
  run bats_pipe cat "$BATS_TEST_DIRNAME/input.json" \| florca run -d "$DEPLOYMENT" --wait
  assert_output --partial 'Output: {"message":"    Hello, World!"}'
}

@test "run $EXAMPLE example workflow with --input" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  # florca run --input ...
  run florca run -d "$DEPLOYMENT" --input '{"text":"Hello, World!","indentation":2}' --wait
  assert_output --partial 'Output: {"message":"  Hello, World!"}'
}

@test "run $EXAMPLE example workflow with --input and empty stdin" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  # florca run --input ... < /dev/null
  run florca run -d "$DEPLOYMENT" --input '{"text":"Hello, World!","indentation":2}' --wait < /dev/null
  assert_output --partial 'Output: {"message":"  Hello, World!"}'
}

@test "run $EXAMPLE example workflow with both stdin and --input" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  # cat input.json | florca run --input ...
  run bats_pipe cat "$BATS_TEST_DIRNAME/input.json" \| florca run -d "$DEPLOYMENT" --input '{"text":"Hello, World!","indentation":2}' --wait
  assert_failure
  assert_output --partial 'Error: Conflicting input sources: stdin and --input'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
