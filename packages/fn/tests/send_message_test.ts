import { expect } from "@std/expect";
import { type PluginContext, sendMessage } from "../mod.ts";

const context = {
  workflowMessageUrl: "http://engine.invalid/1",
  authorizationHeader: "",
} as PluginContext;

const withResponse = async (response: Response, test: () => Promise<void>) => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = () => Promise.resolve(response);
  try {
    await test();
  } finally {
    globalThis.fetch = originalFetch;
  }
};

Deno.test("sendMessage returns the reply", async () => {
  await withResponse(Response.json({ a: 1 }), async () => {
    expect(await sendMessage({}, null, context)).toEqual({ a: 1 });
  });
});

Deno.test("sendMessage fails on a non-2xx status", async () => {
  await withResponse(new Response("boom", { status: 500 }), async () => {
    await expect(sendMessage({}, null, context)).rejects.toThrow(
      "status code 500: boom",
    );
  });
});
