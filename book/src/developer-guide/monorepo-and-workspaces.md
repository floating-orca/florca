# Monorepo & workspaces

There are essentially two workspaces in the _FloatingOrca_ monorepo:

- The Rust workspace with `Cargo.toml` at the root and its crates in the `crates/` directory.
- The Deno workspace with `deno.json` at the root and its packages in the `packages/` directory.

Besides these, the root directory contains a few more files and directories worth mentioning:

- `bacon.toml`: A configuration file for [bacon](https://dystroy.org/bacon/)—a tool that helps with running and watching for changes in Rust projects.
- `compose.yaml`: The Docker Compose configuration file that defines the services required to run _FloatingOrca_.
- `compose.override.yaml`: Overrides certain settings of `compose.yaml` for building _FloatingOrca_ from source.
- `book/`: The book that documents _FloatingOrca_. It is built using [mdBook](https://rust-lang.github.io/mdBook/).
- `dist/`: Configuration files shipped with _FloatingOrca_.
- `docker-entrypoint-initdb.d/`: Configures the `postgres` Docker Compose service to provide not only one database but two: one for the deployer and one for the engine.
- `examples/`: Example workflows that demonstrate how to use _FloatingOrca_.
- `templates/`: Function templates that are used when creating new functions with the CLI.

## Rust crates

The three main components of _FloatingOrca_ are implemented as Rust crates:

- `deployer`: The deployer component, responsible for deploying workflows and functions.
- `engine`: The engine component, responsible for managing the execution of workflows.
- `cli`: The command-line interface that provides the `florca` binary.

All three crates depend on a shared `core` crate that contains common types and utilities used across the components.

To run tests for the Rust crates, you can use the following command in the root directory of the project:

```bash
cargo test
```

## Deno packages

While the Rust `engine` crate manages the execution of workflows, it does not drive the execution of functions. Instead, it spawns a `driver` process for each workflow run. The `driver` is responsible for executing the functions in the workflow, and it is implemented as a Deno package.

Besides the `driver`, there is also a `types` package that defines bindings for the communication between the engine and the driver.

Lastly, the `fn` package provides types and utilities for plugin functions.

To run tests for the Deno packages, you can use the following command in the root directory of the project:

```bash
deno test
```

## The `workflows` directory

The `workflows/` directory is not tracked by Git and can be used to store custom workflows.

You may want to clone your own repository into this directory. This way, you can keep track of your workflows separately from the _FloatingOrca_ codebase.
