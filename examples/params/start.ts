import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { context }: PluginRequestBody,
): Promise<ResponseBody> => ({ payload: context.params });
