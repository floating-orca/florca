import type { RemoteRequestBody, ResponseBody } from "@florca/fn";
import type { InvocationId, LookupEntry } from "@florca/types";
import type { InvokeArgs } from "./run.ts";
import { AUTHORIZATION_HEADER, getEngineUrlForAccessFromKn } from "./mod.ts";

export const invokeKnFunction = async (
  entry: LookupEntry,
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
): Promise<ResponseBody> => {
  const baseUrl = entry.location;
  const funcPort = Deno.env.get("FUNC_PORT") ?? "80";
  const url = `${baseUrl}:${funcPort}`;

  const funcBasicAuth = Deno.env.get("FUNC_BASIC_AUTH");

  const body: RemoteRequestBody = {
    payload: invokeArgs.input,
    context: {
      authorizationHeader: AUTHORIZATION_HEADER,
      id: invocationId,
      params: invokeArgs.params,
      parentId: invokeArgs.parent,
      workflowMessageUrl:
        `${getEngineUrlForAccessFromKn()}/${invokeArgs.runId}`,
    },
  };

  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: funcBasicAuth ? `Basic ${funcBasicAuth}` : "",
    },
    body: JSON.stringify(body),
  });

  return await response.json();
};
