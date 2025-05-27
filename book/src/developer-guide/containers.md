# Containers

## Container images for _FloatingOrca_'s essential components

_FloatingOrca_ is composed of several components. The following are the essential Docker images you should be aware of:

- `caddy`
- `postgres`
- `florca-deployer` with its Dockerfile located at `crates/deployer/Dockerfile`
- `florca-engine` with its Dockerfile located at `crates/engine/Dockerfile`

There is also a `Dockerfile` for the `florca` CLI located at `crates/cli/Dockerfile`. However, this image's purpose is primarily for building the CLI binary, not for running it in a container.

## End-to-end test image

Furthermore, in the `e2e` directory at the root of the repository, there is a `Dockerfile` for building an end-to-end test image, which allows you to run the platform and its dependencies in a single container. This can be useful for testing the functionality of _FloatingOrca_ in a controlled environment and is also used for running an end-to-end test.

### Building the end-to-end test image

To build the end-to-end test image, run the following command from the root of the repository:

```bash
docker build -f e2e/Dockerfile . -t florca-e2e
```

### Running the end-to-end test

To execute the end-to-end test, run:

```bash
docker run --rm florca-e2e
```

This will deploy the `siblings` example workflow, run it, and verify the output.

Alternatively, you can run the following command to build and test in one go:

```bash
./e2e/build-and-test.sh
```

### Interactive shell

The same image can also be used to run an interactive shell with _FloatingOrca_ running in the background:

```bash
docker run --rm -it \
  -v $(pwd)/examples:/usr/src/florca/examples \
  florca-e2e /bin/bash
```

In the interactive shell, you can use the `florca` CLI as usual. For example, to deploy and run the `siblings` example workflow, you can execute:

```bash
florca deploy -w examples/siblings
florca run -d siblings --wait
```

Instead of deploying from within the container, you can also deploy from your host machine by letting the host's `florca` CLI connect to the container.
To do this, let the container expose port 8080 by passing `-p 8080:8080` to the `docker run` command:

```bash
docker run --rm -it \
  -p 8080:8080 \
  florca-e2e /bin/bash
```
