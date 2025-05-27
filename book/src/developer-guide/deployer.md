# Deployer

## API

- `GET /` - List all deployments
- `POST /` - Deploy a workflow
- `GET /{name}` - Fetch a deployment by name
- `DELETE /{name}` - Delete a deployment by name

## AWS Lambda and Knative integration

For AWS Lambda, the deployer makes use of the official AWS SDK for Rust.

For Knative, the deployer wraps Knative's CLI tool `func`.

On both AWS Lambda and Knative, a deployed function is represented by its name, which is derived from the deployment's name and the function's name. For example, if you deploy a workflow as `html` with a function named `start`, the function will be deployed as `html-start`.

## Redeploying functions

Note that AWS Lambda and Knative functions won't get redeployed if their code hasn't changed. Before any redeployment, the deployer first computes a hash of the function's code and compares it to the one stored in the `deployments` table during the last deployment. Only if the hashes differ, the function will be redeployed.

For this reason, you should also avoid connecting different deployer instances to the same AWS account or Knative cluster, as they will not be aware of each other's deployments.

To force the redeployment of functions even if their code hasn't changed according to the computed hash, append the `--force` flag to the `deploy` command of the CLI.

## Plugins & `lookup.json`

While AWS Lambda and Knative functions are deployed remotely and thus the deployer only needs to keep track of their location (e.g., their ARN on AWS or their URL on Knative), plugin functions are executed locally by the engine. This means that the deployer needs to provide the engine with the source code of the plugin functions so that they can be executed.

When the engine asks the deployer for a deployment, the deployer provides a `lookup.json` file that contains the identifiers and locations of AWS Lambda and Knative functions, as well as the source code of the plugin functions. The engine then uses this information to execute the workflow.
