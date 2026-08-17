import { expect } from "@std/expect";
import { dirname, fromFileUrl } from "@std/path";
import { run } from "../lib/run.ts";
import { runWorkflow } from "../lib/mod.ts";
import type { DriverState } from "../lib/driver_state.ts";
import type { InvokeArgs } from "../lib/invoke_args.ts";
import type { DriverArgs, DriverEvent } from "@florca/types";

Deno.env.set("ENGINE_URL", "http://engine.invalid:8080");
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
  inFlightInvocations: new Map(),
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

Deno.test("A failed invocation logs its stack", async () => {
  const logs: { level: string; message: string; data?: { stack?: string } }[] =
    [];
  const loggingState = {
    runId: 1,
    lookupTable: [
      { name: "start", kind: "kn", location: "http://function.invalid" },
    ],
    inFlightInvocations: new Map(),
    eventSink: { addEvent: () => {} },
    invocationLoggerFactory: {
      forInvocation: () => ({
        logEvent: (level: string, message: string, data?: any) => {
          logs.push({ level, message, data });
        },
      }),
    },
  } as unknown as DriverState;

  const originalFetch = globalThis.fetch;
  globalThis.fetch = () => Promise.reject(new Error("boom"));
  try {
    await expect(run(invokeArgs, loggingState)).rejects.toThrow("boom");
  } finally {
    globalThis.fetch = originalFetch;
  }

  const failureLogs = logs.filter((l) => l.message === "Invocation failure");
  expect(failureLogs).toHaveLength(1);
  expect(failureLogs[0].level).toBe("ERROR");
  expect(failureLogs[0].data?.stack).toContain("Error: boom");
  // A stack has frames, not just the error header
  expect(failureLogs[0].data?.stack).toContain("    at ");
});

Deno.test("A sibling still in flight when the workflow fails is recorded as abandoned", async () => {
  const fanOutEvents: DriverEvent[] = [];
  const fanOutState = {
    runId: 1,
    deploymentPath: dirname(fromFileUrl(import.meta.url)),
    lookupTable: [
      { name: "start", kind: "plugin", location: "./fixtures/fan_out.ts" },
      { name: "failing", kind: "kn", location: "http://failing.invalid" },
      { name: "hanging", kind: "kn", location: "http://hanging.invalid" },
    ],
    messageHandlers: new Map(),
    inFlightInvocations: new Map(),
    eventSink: {
      addEvent: (event: DriverEvent) => {
        fanOutEvents.push(event);
      },
    },
    invocationLoggerFactory: {
      forInvocation: () => ({ logEvent: () => {} }),
    },
  } as unknown as DriverState;

  const originalFetch = globalThis.fetch;
  globalThis.fetch = (input: URL | RequestInfo) =>
    String(input).includes("failing.invalid")
      ? Promise.reject(new Error("boom"))
      : new Promise<Response>(() => {});

  try {
    const result = await runWorkflow(
      { entryPoint: "start", input: {}, params: null } as DriverArgs,
      fanOutState,
    );
    expect(result).toMatchObject({ error: { message: "boom" } });
  } finally {
    globalThis.fetch = originalFetch;
  }

  const failures = fanOutEvents.filter((e) => e.type === "invocationFailure");
  const abandoned = failures.filter(
    (e) => (e as { error?: { kind?: string } }).error?.kind === "Abandoned",
  );
  expect(abandoned).toHaveLength(1);
  expect(abandoned[0]).toMatchObject({ functionName: "hanging" });
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
