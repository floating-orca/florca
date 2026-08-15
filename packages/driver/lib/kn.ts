import type { RemoteRequestBody, ResponseBody } from "@florca/fn";
import type { InvocationId, LookupEntry } from "@florca/types";
import type { InvokeArgs } from "./invoke_args.ts";
import type { DriverState } from "./driver_state.ts";
import { getAuthorizationHeader } from "./auth.ts";
import * as env from "./env.ts";

export const invokeKnFunction = async (
  entry: LookupEntry,
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  driverState: DriverState,
): Promise<ResponseBody> => {
  const baseUrl = entry.location;
  const funcPort = env.getKnFuncPort();
  const url = `${baseUrl}:${funcPort}`;

  const funcBasicAuth = env.getKnFuncBasicAuth();

  const body: RemoteRequestBody = {
    payload: invokeArgs.input,
    context: {
      authorizationHeader: getAuthorizationHeader(),
      id: invocationId,
      params: invokeArgs.params,
      parentId: invokeArgs.parent,
      workflowMessageUrl:
        `${env.getEngineUrlForAccessFromKn()}/${driverState.runId}`,
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

  if (!response.ok) {
    const responseBody = await response.text().catch(() => "");
    const error = describeError(responseBody);
    const message = error
      ? `Knative function ${url} failed with error: ${error}`
      : `Knative function ${url} failed with status code: ${response.status}`;
    const invocationLogger = driverState.invocationLoggerFactory
      .forInvocation(invocationId, invokeArgs.functionName);
    invocationLogger.logEvent("ERROR", message);
    if (!error && responseBody) {
      invocationLogger.logEvent("ERROR", responseBody);
    }
    throw new Error(message);
  }

  return await response.json();
};

// Functions may describe their failure as {"error": ...}, like the scaffolded
// templates do.
function describeError(responseBody: string): string | undefined {
  try {
    const { error } = JSON.parse(responseBody);
    return typeof error === "string" ? error : undefined;
  } catch {
    return undefined;
  }
}
