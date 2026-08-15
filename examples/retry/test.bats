#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --wait --show-outputs
  assert_output --partial 'Success: true'
  assert_output --partial 'Output: {"processed":"data"}'
  # The failed attempts show up in the inspection tree
  assert_output --partial 'flaky failure'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
