#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow with --params" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --params '{"factor":2}' --wait
  assert_output --partial 'Output: {"factor":2}'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
