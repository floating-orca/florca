#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --input '{"op":"twice","value":21}' --wait
  assert_output --partial 'Output: 42'

  run florca run -d "$DEPLOYMENT" --input '{"op":"squared","value":6}' --wait
  assert_output --partial 'Output: 36'
}

@test "run $EXAMPLE example workflow with an unmatched key" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --input '{"op":"halved","value":10}' --wait
  assert_output --partial 'Success: false'
  assert_output --partial 'Error: No function for key: halved'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
