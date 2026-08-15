import { expect } from "@std/expect";
import { invokeAwsFunction } from "../lib/aws.ts";
import type { DriverState } from "../lib/driver_state.ts";
import type { InvokeArgs } from "../lib/invoke_args.ts";
import type { LookupEntry } from "@florca/types";

Deno.env.set("ENGINE_URL", "http://engine.invalid:8080");
Deno.env.set("BASIC_AUTH_USERNAME", "user");
Deno.env.set("BASIC_AUTH_PASSWORD", "password");

const entry: LookupEntry = {
  name: "start",
  kind: "aws",
  location: "arn:aws:lambda:eu-central-1:000000000000:function:start",
};

const invokeArgs: InvokeArgs = {
  functionName: "start",
  input: {},
  params: null,
  parent: null,
  predecessor: null,
};

const loggedMessages: string[] = [];

const driverStateReturning = (payload: string, functionError?: string) => {
  const client = {
    send: () =>
      Promise.resolve({
        StatusCode: 200,
        FunctionError: functionError,
        Payload: new TextEncoder().encode(payload),
      }),
  };

  loggedMessages.length = 0;

  return {
    runId: 1,
    lambdaClients: new Map([["eu-central-1", client]]),
    invocationLoggerFactory: {
      forInvocation: () => ({
        logEvent: (_level: unknown, message: string) => {
          loggedMessages.push(message);
        },
      }),
    },
  } as unknown as DriverState;
};

Deno.test("AWS invocation returns the payload", async () => {
  const driverState = driverStateReturning(
    JSON.stringify({ payload: { a: 1 }, next: null }),
  );

  const result = await invokeAwsFunction(
    entry,
    invokeArgs,
    "invocation-id",
    driverState,
  );

  expect(result).toEqual({ payload: { a: 1 }, next: null });
});

Deno.test("AWS invocation reports the error of the function", async () => {
  const driverState = driverStateReturning(
    JSON.stringify({
      errorType: "RuntimeError",
      errorMessage: "boom from lambda",
    }),
    "Unhandled",
  );

  await expect(
    invokeAwsFunction(entry, invokeArgs, "invocation-id", driverState),
  ).rejects.toThrow("RuntimeError: boom from lambda");

  // There are no logs to fall back on, so the message has to be logged anyway.
  expect(loggedMessages.join("\n")).toContain("RuntimeError: boom from lambda");
});

Deno.test("AWS invocation falls back to FunctionError", async () => {
  const driverState = driverStateReturning("not json", "Unhandled");

  await expect(
    invokeAwsFunction(entry, invokeArgs, "invocation-id", driverState),
  ).rejects.toThrow("failed with error: Unhandled");
});
