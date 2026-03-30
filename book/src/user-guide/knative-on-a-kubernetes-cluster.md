# Knative on a Kubernetes cluster

This chapter builds upon the [Self-hosting](./self-hosting.md) chapter and provides instructions for running _FloatingOrca_ Knative functions on a Kubernetes cluster with Knative support. It assumes that your cluster and server are already set up and properly configured.

In this setup, the _FloatingOrca_ engine and deployer run on a cloud server, while the Knative functions execute on the Kubernetes cluster.

## Server-side setup

We'll first need to build _FloatingOrca_ from source, as the Docker images used previously do not support Knative functions.

The following steps illustrate how you could achieve this on the server we set up in the [Self-hosting](./self-hosting.md) chapter:

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

- **(Optional)** If your Knative functions are running on a non-x86 architecture (e.g., ARM-based devices like the NVIDIA Jetson Orin Nano), you may need to adjust the builder used to create the function image.

  To do this, modify the builder configuration in `kn_client.rs`, inside the `generate_func_yaml` function.

  For example, on ARM-based systems, the generated YAML should look like this:

  ```yaml
  specVersion: 0.36.0
  name: {}
  runtime: {}
  created: {}
  build:
    builder: s2i
  ```

- Build the deployer and engine services, and install the CLI:

  ```bash
  cargo build --locked --release --bin florca-deployer
  cargo build --locked --release --bin florca-engine
  ```

- Install kubectl:

  ```bash
  curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
  install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
  rm kubectl
  ```

- Install `func`:

  ```bash
  curl -Lo ./func https://github.com/knative/func/releases/download/knative-v1.18.1/func_linux_amd64
  chmod +x ./func
  mv ./func /usr/local/bin/func
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

  _Note that if your container image registry is not running at `localhost:5001`, set the correct host using the `CONTAINER_REGISTRY` environment variable._

  ```bash
  CONTAINER_REGISTRY=<host> ./target/release/florca-deployer
  ```

- In another terminal, run the engine service:

  ```bash
  target/release/florca-engine
  ```

  _Note that if the ingress port of your Knative cluster is not `80`, you must specify it using the `FUNC_PORT` environment variable:_

  ```bash
  FUNC_PORT=<ingress-port> ./target/release/florca-engine
  ```

  _For Kourier ingress, you can determine the port by running:_
  
  ```bash
  kubectl get svc -n kourier-system
  ```
