#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "kill a single run" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --json
  run_id=$(echo "$output" | jq -r '.runId')
  sleep 1

  run florca ps
  assert_output --partial "$DEPLOYMENT"

  run florca kill "$run_id"
  assert_output "Killed run $run_id"
  sleep 1

  run florca inspect "$run_id" --show-outputs
  assert_output --partial 'Success: false'
  assert_output --partial 'Error: Driver process was killed'
  # The invocation that was in flight when the run was killed
  assert_output --partial 'Abandoned'
}

@test "kill all runs" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  florca run -d "$DEPLOYMENT"
  florca run -d "$DEPLOYMENT"
  sleep 1

  run florca kill --all
  assert_output --partial 'Killed runs'
  # Killed runs stay listed until their processes exit
  sleep 1

  run florca ps
  assert_output '[]'

  run florca kill --all
  assert_output 'No runs killed'
}

@test "kill an unknown run" {
  run florca kill 999999
  assert_failure
  assert_output --partial 'Run 999999 not found'
}

teardown() {
  florca kill -a || true
  florca delete "$DEPLOYMENT" || true
}
