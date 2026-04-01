# Build from source

Instead of using the pre-built binaries and Docker images as described in the [Getting started](./getting-started.md) chapter, you can also build and run _FloatingOrca_ from source.

## Preparation

Clone and enter the repository:

```bash
git clone https://github.com/floating-orca/florca.git
cd florca
```

## Option 1: Build Docker images

Create images for the services and a binary for the CLI.
This is similar to what we distribute in the releases.

1. Build the Docker images:

   ```bash
   docker compose build
   ```

2. Build the CLI:

   ```bash
   docker build -f crates/cli/Dockerfile --output . .
   ```

3. Copy environment-specific files:

   ```bash
   cp dist/src/.env .env
   cp dist/src/Caddyfile Caddyfile
   ```

4. Continue with the [installation](./getting-started.md#installation).

## Option 2: Run natively

Alternatively, closer to the development environment, you can run the services and the CLI natively without Docker.

### Additional prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Deno](https://docs.deno.com/runtime/getting_started/installation/)

### Setup

1. For the databases and the reverse proxy, we would still use Docker. Let's remove any running services and start only the required ones:

   ```bash
   docker compose down
   docker compose up -d caddy postgres
   ```

2. Build the binaries for the deployer and the engine, and install the CLI:

   ```bash
   cargo build --release --bin florca-deployer
   cargo build --release --bin florca-engine
   cargo install --path crates/cli
   ```

3. Copy environment-specific files:

   ```bash
   cp dist/src/.env .env
   cp dist/src/Caddyfile Caddyfile
   ```

4. Perform additional configuration as needed. See [Getting started#Configuration](./getting-started.md#configuration) for details.

5. Now, in one terminal, run the deployer:

   ```bash
   target/release/florca-deployer
   ```

6. In another terminal, run the engine:

   ```bash
   target/release/florca-engine
   ```

   _Important: Run the engine only from the `florca` directory, not anywhere else, as it expects to find a Deno package at `./packages/driver`._

7. Finally, in a third terminal, run the CLI as explained in [Getting started#Run the CLI](./getting-started.md#run-the-cli):

   ```bash
   florca --help
   ```
