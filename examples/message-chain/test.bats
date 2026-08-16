#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  # A plugin messages the workflow handler, which throws. The full cause
  # reaches the sender and fails the run.
  run florca run -d "$DEPLOYMENT" --wait
  assert_output --partial 'Success: false'
  assert_output --partial 'Error: Message failed with status code 500: Driver responded with 500: Error: boom from handler'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
