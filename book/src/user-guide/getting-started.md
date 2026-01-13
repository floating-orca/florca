# Getting started

This chapter will guide you through running _FloatingOrca_ on your local machine using Docker.

## Prerequisites

To get started, you need:

- [Docker](https://docs.docker.com/get-started/get-docker/) and the [Docker Compose plugin](https://docs.docker.com/compose/install/linux/)

## Limitations

So far, _FloatingOrca_ has only been tested on [Fedora Linux](https://fedoraproject.org/) 42, with all instructions tailored to this environment.

In case you are using a different operating system, you may need to adjust the instructions accordingly.
Especially networking often behaves differently on other operating systems, which may require changes to the configuration.
Windows is not supported at all.

Finally, the pre-built Docker images and the CLI binary are only available for Linux on the AMD64 architecture at the moment.
For other platforms, you will need to build the Docker images yourself and compile the CLI binary from source.
See [Build from source](./build-from-source.md) for more information.

## Installation

1. Download the `linux-amd64.tar.gz` asset from the [latest release](https://github.com/floating-orca/florca/releases/latest).

2. Extract the archive to a directory of your choice and enter it:

   ```bash
   tar -xf florca-*-linux-amd64.tar.gz
   ```

   ```bash
   cd florca
   ```

3. (Optional) Link to the CLI binary from a directory in your `PATH`:

   ```bash
   sudo ln -sf "$(pwd)/florca" /usr/local/bin/florca
   # or, if ~/.local/bin is in your PATH
   ln -sf "$(pwd)/florca" ~/.local/bin/florca
   ```

   _Without this step, you will have to run the CLI using `./florca`._

4. Verify the installation:

   ```bash
   florca --version
   ```

## Configuration

The release asset ships with a `.env` file that should—with minor adjustments—work fine locally.

What requires special attention though are the following cases:

- You want to deploy workflow functions to AWS Lambda. This will be covered in the [AWS Lambda functions](./aws-lambda-functions.md) chapter.
- You want to adjust the Basic Authentication credentials.
- You are running rootless Docker.

### Basic Authentication

If you'd like to change the Basic Authentication credentials, you can do so by editing the `BASIC_AUTH_USERNAME` and `BASIC_AUTH_PASSWORD` variables in the `.env` file and adjusting the bcrypt hashes in the `Caddyfile` accordingly. Run `docker run --rm -it caddy:2.10 caddy hash-password` to generate a bcrypt hash for your new password.

### Rootless Docker

If you are running Docker in rootless mode, the special `host-gateway` value in `compose.yaml` is not working as expected.

In rootless mode, `host-gateway` should point to `10.0.2.2` for the containers to be able to communicate with the host.

However, this is currently not the case. _See the ["Incorrect" host-gateway in case with Rootless Docker](https://github.com/moby/moby/issues/47684) GitHub issue for more information._

To fix this, add the `host-gateway-ips` property to the Docker daemon configuration file:

```json
{ "host-gateway-ips": ["10.0.2.2"] }
```

If the file does not exist yet, just run the following command to create it:

```bash
mkdir -p ~/.config/docker
echo '{ "host-gateway-ips": ["10.0.2.2"] }' > ~/.config/docker/daemon.json
```

Furthermore, make sure you have the `DOCKERD_ROOTLESS_ROOTLESSKIT_DISABLE_HOST_LOOPBACK` environment variable set to `false` in `~/.config/systemd/user/docker.service.d/override.conf`:

```conf
[Service]
Environment="DOCKERD_ROOTLESS_ROOTLESSKIT_DISABLE_HOST_LOOPBACK=false"
```

After changing the configuration, reload the systemd configuration and restart the Docker service:

```bash
systemctl --user daemon-reload
systemctl --user restart docker
```

Finally, make sure that the rootless Docker daemon is allowed to bind to port `443` by following the instructions described at <https://docs.docker.com/engine/security/rootless/tips/#exposing-privileged-ports>.

## Connecting to GitHub's container registry

To be able to pull the pre-built Docker images, you'll need to log in to GitHub's container registry.

### Obtain a personal access token (classic)

If you don't have a personal access token (classic) yet, follow these steps to create one:

- Navigate to <https://github.com/settings/tokens>
- Click on the _Generate new token_ dropdown and select _Generate new token (classic)_
- Enter a descriptive note
- Select the `write:packages` and `delete:packages` scopes
- Click _Generate token_ and copy the token

### Log in to the container registry

In a terminal, with your GitHub username and your personal access token (classic) at hand, run:

```bash
docker login ghcr.io
```

### Pull the Docker images

Now you can pull the pre-built Docker images:

```bash
docker pull ghcr.io/floating-orca/deployer:0.7.0
docker pull ghcr.io/floating-orca/engine:0.7.0
```

## Run the services

The following command will start all services in the background:

```bash
docker compose up -d
```

_Skip the `-d` flag to run the containers in the foreground and see their logs._

## Run the CLI

Now that the services are running, you can use the CLI to interact with the system.

### Get help

First, let's see what the CLI can do:

```bash
florca --help
```

_Note that `--help` can be passed to any subcommand._

For completion scripts (for Bash, Zsh, Fish, Elvish, or PowerShell), run:

```bash
florca completions --help
```

### Create, deploy, and run a single-function workflow

To test the setup, take the following steps:

1. Create a new workflow `workflows/getting-started` with a single function `start`:

   ```bash
   florca new --workflow-directory "workflows/getting-started" plugin "start"
   ```

2. Deploy the workflow under the deployment name `getting-started`:

   ```bash
   florca deploy --workflow-directory "workflows/getting-started" "getting-started"
   ```

3. Run the deployed workflow and wait for it to finish:

   ```bash
   florca run --deployment-name "getting-started" --wait
   ```

There should be no errors. Instead, the output should contain the following:

```
Success: true
```

## Example workflows

The release asset you [downloaded earlier](#installation) contains several example workflows in the `examples` directory. A few of them will be covered in later chapters of this user guide. Feel free to explore the others on your own! Some of them include a `README.md` file outlining the workflow and its purpose.
