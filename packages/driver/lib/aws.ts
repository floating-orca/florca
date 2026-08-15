import type { RemoteRequestBody, ResponseBody } from "@florca/fn";
import {
  InvokeCommand,
  type InvokeCommandInput,
  type InvokeCommandOutput,
  LambdaClient,
  LogType,
} from "@aws-sdk/client-lambda";
import { Buffer } from "node:buffer";
import type { InvocationId, LookupEntry } from "@florca/types";
import type { InvokeArgs } from "./invoke_args.ts";
import type { DriverState } from "./driver_state.ts";
import { getAuthorizationHeader } from "./auth.ts";
import * as env from "./env.ts";

function getLambdaClient(region: string, driverState: DriverState) {
  let client = driverState.lambdaClients.get(region);
  if (!client) {
    client = new LambdaClient({ region });
    driverState.lambdaClients.set(region, client);
  }
  return client;
}

export const invokeAwsFunction = async (
  entry: LookupEntry,
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  driverState: DriverState,
): Promise<ResponseBody> => {
  const arn = entry.location;
  const body: RemoteRequestBody = {
    payload: invokeArgs.input,
    context: {
      authorizationHeader: getAuthorizationHeader(),
      id: invocationId,
      params: invokeArgs.params,
      parentId: invokeArgs.parent,
      workflowMessageUrl: `${env.getEngineUrl()}/${driverState.runId}`,
    },
  };

  const region = arn.split(":")[3];
  const client = getLambdaClient(region, driverState);

  const input: InvokeCommandInput = {
    FunctionName: arn,
    InvocationType: "RequestResponse",
    LogType: LogType.Tail,
    Payload: new TextEncoder().encode(JSON.stringify(body)),
  };
  const command = new InvokeCommand(input);

  const response: InvokeCommandOutput = await client.send(command);
  const { FunctionError, Payload, LogResult, StatusCode } = response;

  if (StatusCode !== 200) {
    throw new Error(
      `AWS Lambda function ${arn} failed with status code: ${StatusCode}`,
    );
  }

  const textDecoder = new TextDecoder();
  const payload = textDecoder.decode(Payload);

  let logs: string | undefined;
  if (LogResult) {
    logs = Buffer.from(LogResult, "base64").toString();
  }

  const invocationLogger = driverState.invocationLoggerFactory.forInvocation(
    invocationId,
    invokeArgs.functionName,
  );
  if (FunctionError) {
    const message = `AWS Lambda function ${arn} failed with error: ${
      describeError(payload) ?? FunctionError
    }`;
    invocationLogger.logEvent("ERROR", message);
    if (logs) {
      invocationLogger.logEvent("ERROR", logs);
    }
    throw new Error(message);
  }

  if (logs) {
    invocationLogger.logEvent("DEBUG", logs);
  }

  return JSON.parse(payload);
};

// FunctionError only says whether the function handled the error. The error
// itself is in the payload.
function describeError(payload: string): string | undefined {
  try {
    const { errorType, errorMessage } = JSON.parse(payload);
    if (!errorMessage) {
      return undefined;
    }
    return errorType ? `${errorType}: ${errorMessage}` : errorMessage;
  } catch {
    return undefined;
  }
}
