# Components

This chapter provides details about the various components of _FloatingOrca_.

While the CLI and the deployer are standalone components, the engine requires a Deno package called `driver` to be able to run workflows.
Whenever the engine receives a request to run a workflow, it will spawn a new driver process to execute the workflow.
