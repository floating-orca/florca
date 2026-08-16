import type { PluginRequestBody, ResponseBody } from "@florca/fn";

// Matches a property of the input against a set of cases and runs the
// function of the first match.
export default async (
  { payload, context }: PluginRequestBody,
): Promise<ResponseBody> => {
  const input = payload as Record<string, any>;
  const { match, pass, fns } = context.params;

  const key = input[match];
  const fn = fns[key];
  if (!fn) {
    throw new Error(`No function for key: ${key}`);
  }

  const value = pass ? input[pass] : input;

  return {
    payload: await context.run(fn, value),
    next: context.params.next,
  };
};
