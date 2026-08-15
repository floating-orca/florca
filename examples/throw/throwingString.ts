import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  _requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  throw "Not even an Error!";
};
