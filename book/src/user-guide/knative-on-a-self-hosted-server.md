# Knative on a self-hosted server

This chapter builds upon the [Self-hosting](./self-hosting.md) chapter and provides instructions for running _FloatingOrca_ on a self-hosted server with Knative support. As such, this guide assumes you have already set up and configured the server.

## Server-side setup

We'll first need to build _FloatingOrca_ from source, as the Docker images used previously do not support Knative functions.

The following steps illustrate how you could achieve this on the Hetzner Cloud server we set up in the [Self-hosting](./self-hosting.md) chapter:

- On the server, switch to the home directory:

  ```bash
  cd ~
  ```

- Install Rust:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- Install Deno (and `unzip` if not already installed):

  ```bash
  apt install -y unzip
  curl -fsSL https://deno.land/install.sh | sh
  ```

- Restart your shell.

- Clone the _FloatingOrca_ repository to `florca-from-source` and enter the directory:

  ```bash
  git clone https://github.com/floating-orca/florca.git florca-from-source
  cd florca-from-source
  ```

  _Note that you need to enter your GitHub personal access token instead of your password when prompted._

- Install dependencies that are required for building the project:

  ```bash
  apt install -y build-essential pkg-config libssl-dev
  ```

- Build the deployer and engine services, and install the CLI:

  ```bash
  cargo build --locked --release --bin florca-deployer
  cargo build --locked --release --bin florca-engine
  ```

- Install kind:

  ```bash
  curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.29.0/kind-linux-amd64
  chmod +x ./kind
  mv ./kind /usr/local/bin/kind
  ```

- Install kubectl:

  ```bash
  curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
  install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
  rm kubectl
  ```

- Install `kn`, `kn-quickstart`, and `func`:

  ```bash
  curl -Lo ./kn https://github.com/knative/client/releases/download/knative-v1.18.0/kn-linux-amd64
  chmod +x ./kn
  mv ./kn /usr/local/bin/kn

  curl -Lo ./kn-quickstart https://github.com/knative-extensions/kn-plugin-quickstart/releases/download/knative-v1.18.0/kn-quickstart-linux-amd64
  chmod +x ./kn-quickstart
  mv ./kn-quickstart /usr/local/bin/kn-quickstart

  curl -Lo ./func https://github.com/knative/func/releases/download/knative-v1.18.1/func_linux_amd64
  chmod +x ./func
  mv ./func /usr/local/bin/func
  ```

- Create a Knative cluster:

  ```bash
  kn quickstart kind --registry --kubernetes-version 1.32.8
  ```

  _This could take a very long time. Even if it fails, the cluster could still work fine for our purposes._

- Try deploying a simple test function:

  ```bash
  func create -l node hello
  cd hello
  func deploy -v --registry localhost:5001
  ```

- After the function is deployed, you can run it using:

  ```bash
  curl http://hello.default.127.0.0.1.sslip.io
  ```

  This should yield a response like:

  ```json
  { "query": {} }
  ```

- Remove the test function to continue with the setup:

  ```bash
  func delete hello
  cd ..
  rm -rf hello
  ```

- Copy the `.env` and `Caddyfile` files from the previous installation:

  ```bash
  cp ../florca/.env .
  cp ../florca/Caddyfile .
  ```

- Run the `caddy` and `postgres` services using Docker Compose:

  ```bash
  docker compose up -d caddy postgres
  ```

- Now, in one terminal, run the deployer service:

  ```bash
  target/release/florca-deployer
  ```

- In another terminal, run the engine service:

  ```bash
  target/release/florca-engine
  ```

## Client-side usage

- To test our setup, let's run the following command on the client for deploying the `examples/kn-message` workflow:

  ```bash
  florca deploy -w examples/kn-message
  ```

- Finally, run the workflow:

  ```bash
  florca run -d kn-message --wait
  ```

  Output:

  ```ts
  Success: true
  Output: "SUM OF NUMBERS FROM CHILD INVOCATION: 110"
  Workflow: kn-message (14:22:25.334+02:00) (14:22:33.553+02:00) (PT8.219S)
  + start (14:22:25.721+02:00) (14:22:30.322+02:00) (PT4.601S)
  + child (14:22:25.756+02:00) (14:22:30.293+02:00) (PT4.537S)
  | upper (14:22:30.348+02:00) (14:22:33.516+02:00) (PT3.168S)
  ```
