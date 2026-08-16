import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { context }: PluginRequestBody,
): Promise<ResponseBody> => {
  let done: () => void;
  const finished = new Promise<void>((resolve) => (done = resolve));
  context.onWorkflowMessage(() => {
    setTimeout(() => done(), 100);
    throw new Error("boom from handler");
  });
  await finished;
  return { payload: "finished" };
};
