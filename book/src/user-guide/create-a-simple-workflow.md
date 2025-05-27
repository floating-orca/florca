# Create a simple workflow

Our goal is to create a simple workflow that consists of a few functions and includes conditional branching.
For now, we won't bother about AWS Lambda functions but implement the workflow with plugin functions only.

## A few words on plugin functions

Plugin functions (or simply plugins) are functions that are evaluated directly by the process that drives the workflow.
This means they run in the same Deno runtime and can only be written in TypeScript or JavaScript.

Compared remotely executed AWS Lambda functions, e.g., they ...

- can invoke other functions in a nested fashion,
- can register message handlers to receive messages from other functions,
- can expose HTTP endpoints to receive requests from the outside world.

Because of these capabilities, plugin functions are the most versatile and powerful type of function in the system.

Usually they are used for ...

- custom control flow elements,
- handling data dependencies between functions,
- simple tasks where the overhead of a remote function is not justified.

Finally, they are really easy to write and deploy and thus perfect for getting started.

## Create a new workflow

Let's start off by generating a workflow with a few (plugin) functions:

```bash
florca new -w workflows/simple-example plugin "start"
florca new -w workflows/simple-example plugin "isEven"
florca new -w workflows/simple-example plugin "even"
florca new -w workflows/simple-example plugin "odd"
```

This will create a workflow directory `workflows/simple-example` with the four functions `start`, `isEven`, `even`, and `odd`.
Each function is stored in its own TypeScript (`.ts`) file at the root of the directory.

The idea is to start with a random number and then decide whether it is even or odd.
If it is even, `isEven` will forward the number to `even`, otherwise to `odd`.
The workflow ends with whatever function is called last:

```plaintext
start → isEven → even
               ↳ odd
```

## Implement the functions

Let's chain these functions together by editing the respective files.

<div class="warning">

For proper language support in your IDE, do not place workflows outside the `florca` directory.

In your IDE, just open the root `florca` directory.

This way, the Deno language server (if installed) will pick up the types from the `@florca/fn` package located in the `vendor/fn` directory and referenced by the workspace's `deno.json` file.

Note that this is not necessarily required for your workflow to work, but it will help you in writing the functions, as you'll get autocompletion and type checking for types defined in the `@florca/fn` package.

In case of Visual Studio Code, install [Deno](https://deno.com/) and [Deno for Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=denoland.vscode-deno).

</div>

### `simple-example/start.ts`

```typescript
{{#include ../../../examples/simple-example/start.ts}}
```

This will generate a random number between 1 and 10 and forward it to the `isEven` function.

Each function is ought to return an object with a `payload` and—optionally—a `next` property.
The engine will use the `next` property to determine which function to call next and pass the `payload` to it.

_As you can see, you can just import third-party modules (like `jsr:@std/random`) directly within your function. See Deno's [Modules and dependencies](https://docs.deno.com/runtime/fundamentals/modules/#importing-third-party-modules-and-libraries) for more information._

### `simple-example/isEven.ts`

```typescript
{{#include ../../../examples/simple-example/isEven.ts}}
```

This function checks whether the input is even and forwards it to the `even` or `odd` function accordingly.

Also note how we use `requestBody.context`'s `logEvent` method to log a message from within a plugin function.
The message will show up as follows in the engine's output:

```plaintext
2025-05-30T09:04:10.718139Z  INFO driver: Making decision based on input {
  "input": 3,
  "isEven": false
} run="9" invocation=26 function="isEven"
```

### `simple-example/even.ts`

```typescript
{{#include ../../../examples/simple-example/even.ts}}
```

If the input is even, this function will return a message saying so.

### `simple-example/odd.ts`

```typescript
{{#include ../../../examples/simple-example/odd.ts}}
```

Similarly, if the input is odd, this function will return a message saying so.

## Deploy the workflow

Now that we have implemented the functions, we can deploy the workflow:

```bash
florca deploy -w workflows/simple-example "my-deployment-name"
```

_If no deployment name is provided, the workflow directory name would be used as the deployment name—here: `simple-example`._

## Run the workflow

To run the workflow, call:

```bash
florca run -d "my-deployment-name" --wait --entry-point "start" --show-outputs
```

This will start the workflow and output not only the final result but also the intermediate results of each function (`--show-outputs`).

```ts
Run: 101
Success: true
Output: "The number 3 is odd."
Workflow: my-deployment-name
+ start {"next":"isEven","payload":3}
| isEven {"next":"odd","payload":3}
| odd {"payload":"The number 3 is odd."}
```

_Note that `--entry-point` (or `-e`) is optional and defaults to `start`._

Try to run the workflow a few times to see different results.

## Inspect a workflow run

You can always inspect an arbitrary, finished workflow run by calling:

```bash
florca inspect [OPTIONS] <--latest|RUN_ID>
```

For example:

```bash
florca inspect -io 101
```

The flags `-i` and `-o` are the short forms of `--show-inputs` and `--show-outputs`, respectively.

In this case, you'll see `3` as the input for both `isEven` and `odd`.
The output of the entire workflow equals the output of the function that was called last—`odd` in this case.

```ts
Run: 101
Success: true
Output: "The number 3 is odd."
Workflow: my-deployment-name
+ start {"next":"isEven","payload":3}
| isEven 3 {"next":"odd","payload":3}
| odd 3 {"payload":"The number 3 is odd."}
```

_If you specify `--latest` instead of a specific a `RUN_ID` you'll get the latest run._
