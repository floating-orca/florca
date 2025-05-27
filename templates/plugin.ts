import type { PluginRequestBody, ResponseBody } from "@florca/fn";

type Input = {}; // Define the input type

export default async (
  requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  const input = requestBody.payload as Input;
  const context = requestBody.context;
  return {
    payload: {},
    next: undefined,
  };
};
