import { expect } from "@std/expect";
import map from "../lib/functions/map.ts";
import type { Payload, PluginContext } from "../../fn/mod.ts";

Deno.test("map function maps each input element using a function", async () => {
  const input = [1, 2, 3];
  const context: Partial<PluginContext> = {
    params: {
      fn: "timesTwo",
      reduce: "sum",
    },
    run: async function (
      functionName: string | any,
      payload: Payload,
    ): Promise<Payload> {
      if (functionName !== "timesTwo") {
        throw new Error(
          `Expected function name to be 'timesTwo', got '${functionName}'`,
        );
      }
      return payload * 2;
    },
  };
  const output = await map({
    payload: input,
    context: context as PluginContext,
  });
  expect(output).toMatchObject({
    payload: [2, 4, 6],
    next: "sum",
  });
});
