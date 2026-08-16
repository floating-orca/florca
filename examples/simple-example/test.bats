#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --wait --json
  message=$(echo "$output" | jq -r '.output')
  number=$(echo "$output" | jq '.root.output.payload')
  if [ $((number % 2)) -eq 0 ]; then
    assert_equal "$message" "The number ${number} is even."
  else
    assert_equal "$message" "The number ${number} is odd."
  fi
}

@test "deployment appears in list and disappears after delete" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca list
  assert_output --partial "$DEPLOYMENT"

  florca delete "$DEPLOYMENT"
  run florca list
  refute_output --partial "$DEPLOYMENT"
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
