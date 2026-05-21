# Driver

## API

- `POST /invoke` - Invoke a child function (from within a remote function)
- `POST /` - Send a message to the workflow's message handler
- `POST /:id` - Send a message to a function invocation's message handler
- `GET /` - Retrieve HTML from the workflow's message handler
- `GET /:id` - Retrieve HTML from a function invocation's message handler

## Invocation of individual functions

- For plugin functions, the driver (dynamically) imports the plugin's TypeScript file and invokes the function exported as `default`.
- For AWS Lambda functions, the driver makes use of the official AWS SDK for JavaScript.
- For Knative functions, the driver reads the function's URL from the `functions` table and invokes the function via HTTP.

## Evaluation loop

The following snippet shows the evaluation loop that "drives" the workflow.

After invoking a function, `run` checks if the function returned a `next` value.
If there is no `next`, the branch (or workflow if it's not some child) is complete and the payload is returned.
If there is a `next`, the driver determines the next function to invoke, together with the input and parameters for that function.

Child invocations also enter via the `run` function, with no `predecessor` but with a `parent` set to the invocation ID of the parent function.

```typescript
// "Run" while there is a next function to invoke
export const run = async (
  args: InvokeArgs,
  driverState: DriverState,
): Promise<Payload> => {
  while (true) {
    // Invoke the function
    const [id, response] = await invoke(args, driverState);

    const next = response.next;

    // If there is no next function, return the response
    if (!next) {
      return response.payload;
    }

    // Otherwise, prepare to invoke the next function
    const { functionName, params } = typeof next === "string"
      ? { functionName: next, params: null }
      : { functionName: Object.keys(next)[0], params: next[Object.keys(next)[0]] ?? null };
    args = {
      functionName,
      input: response.payload,
      params,
      parent: null,
      predecessor: id,
    };
  }
};

// "Invoke" a single function and return its response
const invoke = async (
  args: InvokeArgs,
  driverState: DriverState,
): Promise<[InvocationId, ResponseBody]> => {
  const invocationId: InvocationId = crypto.randomUUID();
  const invokeFn = getInvokeFn(args.functionName, driverState.lookupTable);
  const response = await invokeFn(args, invocationId, driverState);
  return [invocationId, response];
};

// The following function is called by `invokeFn`
// when the function to be invoked is a plugin function
async function invokePluginFunction(
  entry: LookupEntry,
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  driverState: DriverState,
): Promise<ResponseBody> {
  const plugin = await import( // import <my-plugin>.ts
    resolve(driverState.deploymentPath, entry.location)
  );
  const body: PluginRequestBody = {
    payload: invokeArgs.input,
    context: {
      id: invocationId,
      params: invokeArgs.params,
      parentId: invokeArgs.parent,
      run: (fn: string | any, payload: Payload) => {
        // This is the context.run method for invoking
        // a function from within a plugin function
        let functionName;
        let params;
        if (typeof fn === "string") {
          functionName = fn;
        } else {
          functionName = Object.keys(fn)[0];
          params = fn[functionName];
        }
        const invokeArgs: InvokeArgs = {
          functionName,
          input: payload,
          params: params ?? null,
          parent: invocationId,
          predecessor: null,
        };
        return run(invokeArgs, driverState);
      },
      // ...
    },
  };
  const response = await plugin.default(body);
  return response;
}
```

_Note that only the root function of a child workflow has a `parent` set. Subsequent functions have a `predecessor` set instead._

## IPC

The driver communicates with the engine primarily via HTTP:

- The driver sends all events (invocations and logs) in batches to the engine's `POST /{run}/events` endpoint.
- Before the driver exits, it flushes any remaining events to ensure all invocation and log data has been delivered, then sends a separate completion request to the engine's `POST /{run}/complete` endpoint to signal the workflow result.
