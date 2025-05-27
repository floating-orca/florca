# Expose an HTTP endpoint

<div class="warning">

**This is very much a proof of concept.**

Especially the injection of the authorization header into the rendered HTML is far from ideal.
There are better ways to handle this, but they are not yet implemented.

</div>

In the [previous chapter](./messaging.md), we learned how to register a message handler to receive messages from other functions. We've also seen how to register a workflow-level message handler using `context.onWorkflowMessage`.

In this chapter, we'll learn how to use such a workflow-level message handler to expose an HTTP endpoint that can be used to interact with the workflow from the outside.

You may have noticed that when you run a workflow, the engine prints a URL to the console. This URL is the endpoint that you can use to interact with a registered workflow-level message handler.

You can use this to send some JSON data to the workflow, for example, or to render an HTML form, as shown in the example below.

Currently, there are separate endpoints for sending JSON and retrieving HTML, respectively.

- `POST /{run}`: Takes a JSON payload and responds with a JSON payload.
- `GET /{run}`: Takes no payload or parameters and responds with an HTML payload.

_Side note: If you'd like to request HTML from a specific function, you can send a request to `GET /{run}/{invocationNumber}`. Just note that this will target the message handler set via `context.onMessage`, not the workflow-level message handler set via `context.onWorkflowMessage`._

## HTML

The following function demonstrates how to expose an HTTP endpoint that renders an HTML form and waits for the form to be submitted. The workflow-level message handler is used to render the form, while the function-level message handler waits for the form to be submitted.

```typescript
{{#include ../../../examples/html/start.ts}}
```

_Note how the `renderHtml` function inserts the function's invocation ID into the target URL in order to let the browser send the form data to the correct endpoint—the function-level message handler._

## Kill running workflows

Especially when dealing with workflows that wait for messages, it's important to be able to stop them.

You can see what workflows are currently running by running the `ps` command:

```bash
florca ps
```

To stop a running workflow, use the `kill` command:

```bash
florca kill <RUN_ID>
```

You can also kill all running workflows by calling:

```bash
florca kill --all
```
