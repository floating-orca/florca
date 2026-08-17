import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  _requestBody: PluginRequestBody,
): Promise<ResponseBody> => ({
  payload: null,
  next: {
    parallel: {
      fns: ["failing", "slow"],
    },
  },
});
