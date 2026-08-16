import {
  type PluginRequestBody,
  type ResponseBody,
  sendMessageToWorkflow,
} from "@florca/fn";

export default async (
  { context }: PluginRequestBody,
): Promise<ResponseBody> => {
  return { payload: await sendMessageToWorkflow("hi", context) };
};
