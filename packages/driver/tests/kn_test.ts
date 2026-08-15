import { expect } from "@std/expect";
import { invokeKnFunction } from "../lib/kn.ts";
import type { DriverState } from "../lib/driver_state.ts";
import type { InvokeArgs } from "../lib/invoke_args.ts";
import type { LookupEntry } from "@florca/types";

Deno.env.set("ENGINE_URL_FOR_ACCESS_FROM_KN", "http://engine.invalid:8080");
Deno.env.set("BASIC_AUTH_USERNAME", "user");
Deno.env.set("BASIC_AUTH_PASSWORD", "password");

const entry: LookupEntry = {
  name: "start",
  kind: "kn",
  location: "http://function.invalid",
};

const invokeArgs: InvokeArgs = {
  functionName: "start",
  input: {},
  params: null,
  parent: null,
  predecessor: null,
};

const loggedMessages: string[] = [];

const driverState = {
  runId: 1,
  invocationLoggerFactory: {
    forInvocation: () => ({
      logEvent: (_level: unknown, message: string) => {
        loggedMessages.push(message);
      },
    }),
  },
} as unknown as DriverState;

const withResponse = async (response: Response, test: () => Promise<void>) => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = () => Promise.resolve(response);
  loggedMessages.length = 0;
  try {
    await test();
  } finally {
    globalThis.fetch = originalFetch;
  }
};

Deno.test("Knative invocation returns the response body", async () => {
  const response = Response.json({ payload: { a: 1 }, next: null });

  await withResponse(response, async () => {
    const result = await invokeKnFunction(
      entry,
      invokeArgs,
      "invocation-id",
      driverState,
    );
    expect(result).toEqual({ payload: { a: 1 }, next: null });
  });
});

Deno.test("Knative invocation fails on a non-2xx status", async () => {
  // The body is a valid response body, so without a status check it would be
  // mistaken for a successful invocation.
  const response = Response.json({ payload: { error: "boom" }, next: null }, {
    status: 500,
  });

  await withResponse(response, async () => {
    await expect(
      invokeKnFunction(entry, invokeArgs, "invocation-id", driverState),
    ).rejects.toThrow("failed with status code: 500");
  });

  expect(loggedMessages.join("\n")).toContain("failed with status code: 500");
  expect(loggedMessages.join("\n")).toContain("boom");
});

Deno.test("Knative invocation reports the error of the function", async () => {
  const response = Response.json({ error: "RuntimeError: boom" }, {
    status: 500,
  });

  await withResponse(response, async () => {
    await expect(
      invokeKnFunction(entry, invokeArgs, "invocation-id", driverState),
    ).rejects.toThrow("failed with error: RuntimeError: boom");
  });

  expect(loggedMessages.join("\n")).toContain(
    "failed with error: RuntimeError: boom",
  );
});
