#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --wait
  assert_output --partial 'Success: false'
  assert_output --partial "Error: Function 'nonExistentFunction' not found"
}

@test "run a workflow of an unknown deployment" {
  run florca run -d does-not-exist-test --wait
  assert_failure
  assert_output --partial 'Deployment does-not-exist-test not found'
}

@test "run an unknown entry point" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" -e nope --wait
  assert_failure
  assert_output --partial 'Entry point not found: nope'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
