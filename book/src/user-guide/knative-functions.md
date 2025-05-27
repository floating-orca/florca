# Knative functions

As an alternative to AWS Lambda functions you can also use Knative functions.

Currently, support for Knative functions is limited and rather meant for testing purposes, e.g. to avoid risk of incurring costs when using AWS Lambda functions.

This chapter describes how to set up a local Knative cluster and deploy a Knative function to it.

## Additional prerequisites

<div class="warning">

Note that Knative functions can currently only be deployed and invoked when the engine and the deployer are running natively, not on Docker Compose, as neither the engine nor the deployer images have a Knative client installed and configured. Furthermore, the way Knative functions are built right now, the deployer container would need to be able to itself invoke Docker commands to build the function image.

</div>

- Engine and deployer running natively
  - See [Build from source#Option 2: Run natively](./build-from-source.md#option-2-run-natively) for instructions on how to do this
- [Kind](https://kind.sigs.k8s.io/docs/user/quick-start/#installation)
  - No need to create a cluster, as the `kn quickstart` command will do that for you
- [kubectl](https://kubernetes.io/docs/tasks/tools/#kubectl)
  - No need to configure it, as the `kn quickstart` command will do that for you
- The following Knative binaries in your `PATH`:
  - [`kn`](https://knative.dev/docs/install/quickstart-install/#install-the-knative-cli)
  - [`kn-quickstart`](https://knative.dev/docs/install/quickstart-install/#install-the-knative-quickstart-plugin)
  - [`func`](https://knative.dev/docs/functions/install-func/)

## Setup

```bash
kn quickstart kind --registry --kubernetes-version 1.32.8
```

This command will create a local Kubernetes cluster using Kind and install Knative on it.
It will also create a local container image registry running on `localhost:5001`, to which Knative function images will be pushed.

### Test Knative

Let's create a simple Knative function and deploy it to the local cluster without _FloatingOrca_, just to see whether the Knative setup is working:

```bash
func create -l node hello
cd hello
func deploy -v --registry localhost:5001
```

Check out the [Troubleshooting](#troubleshooting) section if you run into issues.

## Deploying Knative functions with _FloatingOrca_

### Create a new workflow

Let's start off by generating a workflow with a few functions:

```bash
florca new -w workflows/kn-example function -p kn -r "python" "start"
```

This will create a workflow directory `workflows/kn-example` with a single function `start` (Knative function using the `python` runtime).

### Deploy the workflow

```bash
florca deploy -w workflows/kn-example
```

### Run the workflow

```bash
florca run -d "kn-example" --wait --show-outputs
```

## Communication between Knative functions and plugin functions

If you are running Docker in rootless mode, add the following line to your `.env.local` file:

```
ENGINE_URL_FOR_ACCESS_FROM_KN=http://engine.10-0-2-2.sslip.io:8080
```

This will allow Knative functions to access the engine running on the host machine.

The default setting in `.env` is:

```
ENGINE_URL_FOR_ACCESS_FROM_KN=http://engine.172-17-0-1.sslip.io:8080
```

This should work for most users running Docker in regular (non-rootless) mode.
However, in some cases, the IP address may not be `172.17.0.1`, but `172.18.0.1`, `172.19.0.1`, or similar (depending on the Docker network that is used).

See the `examples/messaging/kn-py` directory for an example workflow that illustrates the communication between Knative functions and plugin functions.

## Troubleshooting

Knative is a complex system and can be tricky to get running.

When familiar with Kubernetes, use `kubectl`, [`k9s`](https://k9scli.io/), or any other interface to check the status of the cluster.

If the deployment fails, pods could not be started, you cannot access the internet from within the cluster, or something else goes wrong, you can try the following:

- Run the command that failed again.

- Try restarting the Docker daemon before running the command again:

  ```bash
  sudo systemctl restart docker
  ```

  or, if you are running Docker in rootless mode:

  ```bash
  systemctl --user restart docker
  ```

- Try to delete the Kind cluster and start over, but this time with the firewall and (if applicable) SELinux disabled.

  To delete the Kind cluster, run:

  ```bash
  kind delete cluster
  ```

  On Fedora, you can temporarily disable the firewall and SELinux by running:

  ```bash
  sudo systemctl stop firewalld
  sudo setenforce 0
  ```

  Then, run the `kn quickstart` command again:

  ```bash
  kn quickstart kind --registry --kubernetes-version 1.32.8
  ```

- When pods of the `knative-serving` namespace fail to start because of some version incompatibility (indicated in the logs), you can try to specify a different Kubernetes version to use with Kind:

  ```bash
  kn quickstart kind --registry --kubernetes-version 1.x.y
  ```

  *See [`kindest/node`](https://hub.docker.com/r/kindest/node/) for available versions.*
