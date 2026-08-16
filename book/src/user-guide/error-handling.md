# Error handling

_FloatingOrca_ has no dedicated retry or catch constructs.
A failing child function simply throws in its parent, so errors are handled with plain `try`/`catch`.

## Errors propagate to the parent

When a child function invoked with `context.run` throws, the parent's `context.run` call rejects with that error.
This works the same for remote functions: a failing AWS Lambda or Knative function rejects the `context.run` call of its parent.

## Retrying a flaky function

Because a child invocation is an ordinary expression that throws on failure, retrying is a plain loop:

```typescript
{{#include ../../../examples/retry/start.ts}}
```

The child function fails twice before succeeding:

```typescript
{{#include ../../../examples/retry/fetchData.ts}}
```

The failed attempts still show up in the inspection tree, so even a successful run keeps a record of its retries.

## Uncaught errors fail the run

When a function throws and nothing catches the error, the run fails.
The CLI reports the error along with `Success: false`:

```plaintext
Success: false
Error: What a terrible failure!
```

The `examples/throw` workflow demonstrates this.

## Throwing message handlers

A message handler that throws fails the sender, not the run.
The sender's `sendMessage` call fails with the handler's error message, and the workflow continues, as the `examples/message-throw` workflow demonstrates.
