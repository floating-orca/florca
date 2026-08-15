import type { PluginRequestBody, ResponseBody } from "@florca/fn";

const maxAttempts = 3;

export default async (
  { payload, context }: PluginRequestBody,
): Promise<ResponseBody> => {
  for (let attempt = 1; ; attempt++) {
    try {
      return {
        payload: await context.run("fetchData", payload),
        next: "processData",
      };
    } catch (e) {
      if (attempt === maxAttempts) throw e;
    }
  }
};
