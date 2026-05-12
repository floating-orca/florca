#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

EXPECTED='Output: "iVBORw0KGgoAAAANSUhEUgAAAQAAAAEACAIAAADTED8xAAACyElEQVR4nOzTwQkAMAzEsBS6/' # (truncated)

@test "run $EXAMPLE example workflow" {
  florca deploy -w "$BATS_TEST_DIRNAME" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --wait
  assert_output --partial "$EXPECTED"
}

teardown() {
  florca delete "$DEPLOYMENT" || true
}
