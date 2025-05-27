import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  const context = requestBody.context;
  const promises = [];
  for (const fn of context.params.fns) {
    promises.push(context.run(fn, requestBody.payload));
  }
  return {
    payload: await Promise.all(promises),
    next: context.params.reduce,
  };
};
