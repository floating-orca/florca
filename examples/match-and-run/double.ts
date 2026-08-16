import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { payload }: PluginRequestBody,
): Promise<ResponseBody> => ({ payload: (payload as number) * 2 });
