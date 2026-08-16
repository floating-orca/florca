# Run a child function

In the previous chapter, we learned how to instruct the engine what function to run next by specifying the `next` property in the response object.

Another way to control the flow of the workflow is to run a child function from a parent function. We'll see later how this can be useful when implementing more complex workflows.

<div class="warning">

Note that the `context.run` method described below is available to plugin functions only.
Remote functions like [AWS Lambda functions](./aws-lambda-functions.md) can still invoke child functions through the engine's `/invoke` endpoint.
There is no packaged helper for this yet, but the `examples/remote-invocation` workflow contains small `run` helpers for AWS Lambda and Knative functions.

</div>

To run a child function, use the `run` method available on the plugin's `requestBody.context` object. The method takes two arguments: the name of the function to run and the payload to pass to it.

## Example

Here is an example of a plugin that runs a child function called `toUpper` on a string "Hello, world!", awaiting the result and returning it:

```typescript
{{#include ../../../examples/to-upper/start.ts}}
```

The `toUpper` function could be implemented as follows:

```typescript
{{#include ../../../examples/to-upper/toUpper.ts}}
```

Since the child function `toUpper` does not return any `next`, this branch of the workflow will end here and return the uppercase string to the parent function.

## Control flow

<div class="warning">

When the invoked child function returns a `next`, we're rather dealing with a "child workflow" than a simple child function invocation.

</div>

To some extent, the invocation of a child function can be seen as the invocation of a child workflow, where the child workflow consists of a single function.

If a child function returns a `next`, the workflow branch continues with the specified function and payload.

Only when a function does not return a `next`, the branch ends, and control is returned to the parent function, with the Promise of `context.run` resolving to the result of the branch's last function.

![Illustration of the control flow when running a child workflow](./control-flow.excalidraw.png)
