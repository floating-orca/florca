#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --json
  run_id=$(echo "$output" | jq -r '.runId')
  sleep 1

  # The sender sees the cause of the throwing handler
  run florca message --run-id "$run_id" '"x"'
  assert_failure
  assert_output --partial 'boom from handler'
  sleep 1

  # The throwing handler does not block the run
  run florca inspect "$run_id"
  assert_output --partial 'Success: true'
  assert_output --partial 'Output: "finished"'
}

teardown() {
  florca kill -a
  florca delete "$DEPLOYMENT" || true
}
