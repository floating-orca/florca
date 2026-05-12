import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  requestBody: PluginRequestBody,
): Promise<ResponseBody> => {
  const { payload } = requestBody;
  let { indentation, text } = payload as { indentation: number; text: string };
  for (let i = 0; i < indentation; i++) {
    text = ` ${text}`;
  }
  return {
    payload: { message: text },
  };
};
