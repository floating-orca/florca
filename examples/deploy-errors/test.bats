#!/usr/bin/env bats

bats_load_library bats-support
bats_load_library bats-assert

EXAMPLE=$(basename "$BATS_TEST_DIRNAME")
DEPLOYMENT="${EXAMPLE}-test"

setup() {
  temp_dir=$(mktemp -d)
  cp -r "$BATS_TEST_DIRNAME"/* "$temp_dir"
}

@test "a failed deploy keeps the previous deployment" {
  florca deploy -w "$temp_dir" "$DEPLOYMENT"
  run florca run -d "$DEPLOYMENT" --wait
  assert_output --partial 'Output: "v1"'

  # Break the workflow: new output plus an invalid function config
  sed -i 's/"v1"/"v2"/' "$temp_dir/start.ts"
  mkdir "$temp_dir/broken"
  echo "not valid toml =" > "$temp_dir/broken/function.toml"
  run florca deploy -w "$temp_dir" "$DEPLOYMENT"
  assert_failure
  assert_output --partial 'function.toml'

  # The previous deployment still runs
  run florca run -d "$DEPLOYMENT" --wait
  assert_output --partial 'Output: "v1"'
}

@test "underscores are rejected for Knative function names" {
  mkdir "$temp_dir/my_func"
  printf 'provider = "kn"\nruntime = "python"\n' > "$temp_dir/my_func/function.toml"
  run florca deploy -w "$temp_dir" "$DEPLOYMENT"
  assert_failure
  assert_output --partial 'Knative function name my_func must not contain underscores'
}

@test "underscores are rejected in deployment names with Knative functions" {
  mkdir "$temp_dir/knfunc"
  printf 'provider = "kn"\nruntime = "python"\n' > "$temp_dir/knfunc/function.toml"
  run florca deploy -w "$temp_dir" foo_bar_test
  assert_failure
  assert_output --partial 'must not contain underscores when deploying Knative functions'
}

teardown() {
  florca delete "$DEPLOYMENT" || true
  rm -rf "$temp_dir"
}
