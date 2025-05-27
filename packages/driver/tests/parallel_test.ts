import { expect } from "@std/expect";
import parallel from "../lib/functions/parallel.ts";
import type { Payload, PluginContext } from "../../fn/mod.ts";

Deno.test("parallel function applies multiple functions in parallel", async () => {
  const input = "Test";
  const context: Partial<PluginContext> = {
    params: {
      fns: ["upper", "lower"],
      reduce: "concat",
    },
    run: async function (
      functionName: string | any,
      payload: Payload,
    ): Promise<Payload> {
      if (functionName === "upper") {
        return payload.toUpperCase();
      } else if (functionName === "lower") {
        return payload.toLowerCase();
      } else {
        throw new Error(
          `Expected function name to be 'upper' or 'lower', got '${functionName}'`,
        );
      }
    },
  };
  const output = await parallel({
    payload: input,
    context: context as PluginContext,
  });
  expect(output).toMatchObject({
    payload: ["TEST", "test"],
    next: "concat",
  });
});
