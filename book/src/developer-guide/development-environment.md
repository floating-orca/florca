# Development environment

## Additional prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Deno](https://docs.deno.com/runtime/getting_started/installation/)

## Setup

1. Clone and enter the repository:

   ```bash
   git clone https://github.com/floating-orca/florca.git
   cd florca
   ```

2. Copy environment-specific files:

   ```bash
   cp dist/src/.env .env
   cp dist/src/Caddyfile Caddyfile
   ```

3. Perform additional configuration as needed. See [Getting started#Configuration](../user-guide/getting-started.md#configuration) for details.

## Run required services

1. Remove any running services:

   ```bash
   docker compose down
   ```

2. Start only the required services:

   ```bash
   docker compose up -d caddy postgres
   ```

## Run the debug builds of the servers

1. In one terminal, run the deployer:

   ```bash
   cargo run --bin florca-deployer
   ```

2. In another terminal, run the engine:

   ```bash
   cargo run --bin florca-engine
   ```

### Watching for changes

Using [bacon](https://dystroy.org/bacon/), you won't have to restart the servers manually after making changes to the source code.

If you haven't installed bacon yet, do so by running:

```bash
cargo install --locked bacon
```

#### Run the servers using bacon

In one terminal, run the deployer:

```bash
bacon run-deployer
```

In another terminal, run the engine:

```bash
bacon run-engine
```

## Run the CLI debug build

```bash
cargo run --bin florca
```

Pass any arguments to the CLI after `--`. For example, to get help, run:

```bash
cargo run --bin florca -- --help
```

## Build the book

The documentation is built using [mdBook](https://rust-lang.github.io/mdBook/) and can be found in the `book/` directory.

Install mdBook using Cargo:

```bash
cargo install mdbook
```

Then, to build the book, run:

```bash
mdbook build book
```

You can find the built book in `book/book/`.

To serve the book locally and watch for changes, run:

```bash
mdbook serve book
```
