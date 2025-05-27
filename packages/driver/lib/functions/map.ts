import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  const elements: any[] = requestBody.payload;
  const context = requestBody.context;
  const promises = [];
  for (const element of elements) {
    promises.push(context.run(context.params.fn, element));
  }
  return {
    payload: await Promise.all(promises),
    next: context.params.reduce,
  };
};
