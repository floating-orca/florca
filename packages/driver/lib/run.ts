import "@std/dotenv/load";
import { invokeAwsFunction } from "./aws.ts";
import { invokeKnFunction } from "./kn.ts";
import type { Payload, ResponseBody } from "@florca/fn";
import type {
  DriverEvent,
  FunctionName,
  InvocationId,
  LookupEntry,
} from "@florca/types";
import type { InvokeArgs } from "./invoke_args.ts";
import { invokePluginFunction } from "./plugin.ts";
import type { DriverState } from "./driver_state.ts";

export class FunctionNotFoundError extends Error {
  constructor(functionName: FunctionName) {
    super(`Function '${functionName}' not found`);
    this.name = "FunctionNotFoundError";
  }
}

// Functions are not guaranteed to throw an Error.
export function describeThrown(e: unknown): { kind: string; message: string } {
  return e instanceof Error
    ? { kind: e.constructor.name, message: e.message }
    : { kind: "UnknownError", message: String(e) };
}

// "Run" while there is a next function to invoke
export const run = async (
  args: InvokeArgs,
  driverState: DriverState,
): Promise<Payload> => {
  while (true) {
    // Invoke the function
    const [id, response] = await invoke(args, driverState);

    const next = response.next;

    // If there is no next function, return the response
    if (!next) {
      return response.payload;
    }

    // Otherwise, prepare to invoke the next function
    // deno-fmt-ignore
    const { functionName, params } = typeof next === "string"
      ? { functionName: next, params: null }
      : { functionName: Object.keys(next)[0], params: next[Object.keys(next)[0]] ?? null };
    args = {
      functionName,
      input: response.payload,
      params,
      parent: null,
      predecessor: id,
    };
  }
};

// "Invoke" a single function and return its response
const invoke = async (
  args: InvokeArgs,
  driverState: DriverState,
): Promise<[InvocationId, ResponseBody]> => {
  const invocationId: InvocationId = crypto.randomUUID();
  const startTime = Temporal.Now.instant();
  logInvocationStart(args, invocationId, driverState);
  try {
    const invokeFn = getInvokeFn(args.functionName, driverState.lookupTable);
    const response = await invokeFn(args, invocationId, driverState);
    const endTime = Temporal.Now.instant();
    driverState.eventSink.addEvent(
      newSuccessEvent(args, invocationId, startTime, response, endTime),
    );
    return [invocationId, response];
  } catch (e) {
    driverState.eventSink.addEvent(
      newFailureEvent(args, invocationId, startTime, describeThrown(e)),
    );
    throw e;
  }
};

type InvokeFn = (
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  driverState: DriverState,
) => Promise<ResponseBody>;

function getInvokeFn(
  functionName: FunctionName,
  lookupTable: LookupEntry[],
): InvokeFn {
  const entry = findLookupEntry(functionName, lookupTable);
  switch (entry.kind) {
    case "aws":
      return (invokeArgs, invocationId, driverState) =>
        invokeAwsFunction(entry, invokeArgs, invocationId, driverState);
    case "kn":
      return (invokeArgs, invocationId, driverState) =>
        invokeKnFunction(entry, invokeArgs, invocationId, driverState);
    case "plugin":
      return (invokeArgs, invocationId, driverState) =>
        invokePluginFunction(entry, invokeArgs, invocationId, driverState);
    default:
      throw new Error(`Unknown function type: ${entry}`);
  }
}

function logInvocationStart(
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  driverState: DriverState,
) {
  const invocationLogger = driverState.invocationLoggerFactory.forInvocation(
    invocationId,
    invokeArgs.functionName,
  );
  invocationLogger.logEvent("DEBUG", "Invocation start", {
    input: invokeArgs.input,
    params: invokeArgs.params,
  });
}

function newSuccessEvent(
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  startTime: Temporal.Instant,
  response: ResponseBody,
  endTime: Temporal.Instant,
): DriverEvent {
  return {
    type: "invocationSuccess",
    id: invocationId,
    parent: invokeArgs.parent,
    predecessor: invokeArgs.predecessor,
    functionName: invokeArgs.functionName,
    input: invokeArgs.input ?? null,
    params: invokeArgs.params ?? null,
    output: response ?? null,
    startTime: startTime.toString(),
    endTime: endTime.toString(),
  };
}

function newFailureEvent(
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
  startTime: Temporal.Instant,
  error: { kind: string; message: string },
): DriverEvent {
  return {
    type: "invocationFailure",
    id: invocationId,
    parent: invokeArgs.parent,
    predecessor: invokeArgs.predecessor,
    functionName: invokeArgs.functionName,
    input: invokeArgs.input ?? null,
    params: invokeArgs.params ?? null,
    startTime: startTime.toString(),
    error,
  };
}

function findLookupEntry(
  functionName: string,
  lookupTable: LookupEntry[],
): LookupEntry {
  const entry = lookupTable.find((f) => f.name === functionName);
  if (!entry) {
    throw new FunctionNotFoundError(functionName);
  }
  return entry;
}
