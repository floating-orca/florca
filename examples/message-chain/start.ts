import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { context }: PluginRequestBody,
): Promise<ResponseBody> => {
  context.onWorkflowMessage(() => {
    throw new Error("boom from handler");
  });
  return { payload: await context.run("poke", null) };
};
