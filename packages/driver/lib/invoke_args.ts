import type { FunctionName, InvocationId, JsonValue } from "@florca/types";

export type InvokeArgs = {
  functionName: FunctionName;
  input: JsonValue;
  params: JsonValue;
  parent: InvocationId | null;
  predecessor: InvocationId | null;
};
