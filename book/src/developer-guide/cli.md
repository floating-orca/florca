# CLI

The `cli` crate provides the `florca` binary—a command-line interface for creating new functions and for interacting with the deployer and the engine.

It's implemented in Rust and communicates with other components via HTTP.

The various commands and their options are defined in `crates/cli/src/cli.rs` using the [`clap`](https://docs.rs/clap/latest/clap/) library.
If you want to add a new command, make sure you adapt `crates/cli/src/lib.rs` accordingly.

## Note on `florca run --wait`

When you run `florca run` without `--wait`, the CLI will only print the ID of the workflow run but not wait for the run to complete.
If you want to wait for the run to complete, you can use `--wait` or `-w`. Just note that the CLI will only check every second whether the run is complete or not.
