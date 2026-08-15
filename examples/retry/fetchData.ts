import type { ResponseBody } from "@florca/fn";

// Fails twice before succeeding. Module state persists for the whole run.
let attempts = 0;

export default async (): Promise<ResponseBody> => {
  attempts++;
  if (attempts < 3) {
    throw new Error("flaky failure");
  }
  return { payload: "data" };
};
