import { expect } from "@std/expect";
import { run } from "../lib/run.ts";
import { runWorkflow } from "../lib/mod.ts";
import type { DriverState } from "../lib/driver_state.ts";
import type { InvokeArgs } from "../lib/invoke_args.ts";
import type { DriverArgs, DriverEvent } from "@florca/types";

Deno.env.set("ENGINE_URL_FOR_ACCESS_FROM_KN", "http://engine.invalid:8080");
Deno.env.set("BASIC_AUTH_USERNAME", "user");
Deno.env.set("BASIC_AUTH_PASSWORD", "password");

const invokeArgs: InvokeArgs = {
  functionName: "start",
  input: {},
  params: null,
  parent: null,
  predecessor: null,
};

const events: DriverEvent[] = [];

const driverState = {
  runId: 1,
  lookupTable: [
    { name: "start", kind: "kn", location: "http://function.invalid" },
  ],
  eventSink: {
    addEvent: (event: DriverEvent) => {
      events.push(event);
    },
  },
  invocationLoggerFactory: {
    forInvocation: () => ({ logEvent: () => {} }),
  },
} as unknown as DriverState;

Deno.test("A failure that is not an Error is still reported", async () => {
  events.length = 0;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = () => Promise.reject("boom");

  try {
    await expect(run(invokeArgs, driverState)).rejects.toEqual("boom");
  } finally {
    globalThis.fetch = originalFetch;
  }

  const failures = events.filter((e) => e.type === "invocationFailure");
  expect(failures).toHaveLength(1);
  expect(failures[0]).toMatchObject({
    error: { kind: "UnknownError", message: "boom" },
  });
});

Deno.test("A workflow failing with something that is not an Error is reported", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = () => Promise.reject("boom");

  try {
    // A rethrow here would kill the driver before its events are flushed.
    const result = await runWorkflow(
      { entryPoint: "start", input: {}, params: null } as DriverArgs,
      driverState,
    );
    expect(result).toEqual({
      error: { kind: "UnknownError", message: "boom" },
    });
  } finally {
    globalThis.fetch = originalFetch;
  }
});
