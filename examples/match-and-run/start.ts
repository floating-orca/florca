import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  { payload }: PluginRequestBody,
): Promise<ResponseBody> => ({
  payload,
  next: {
    matchAndRun: {
      match: "op",
      pass: "value",
      fns: { twice: "double", squared: "square" },
    },
  },
});
