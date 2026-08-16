import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { payload }: PluginRequestBody,
): Promise<ResponseBody> => {
  await new Promise((resolve) =>
    setTimeout(resolve, (payload as number) ?? 30000)
  );
  return { payload: "done" };
};
