import "@std/dotenv/load";
import { invokeAwsFunction } from "./aws.ts";
import { invokeKnFunction } from "./kn.ts";
import type { Payload, ResponseBody } from "@florca/fn";
import type {
  DeploymentName,
  FunctionName,
  JsonValue,
  LookupEntry,
  RunId,
} from "@florca/types";
import { invokePluginFunction } from "./plugin.ts";
import { logEvent } from "./mod.ts";

export class FunctionNotFoundError extends Error {
  constructor(functionName: FunctionName) {
    super(`Function '${functionName}' not found`);
    this.name = "FunctionNotFoundError";
  }
}

export type InvokeArgs = {
  runId: RunId;
  deploymentName: DeploymentName;
  deploymentPath: string;
  functionName: FunctionName;
  input: JsonValue;
  params: JsonValue;
  parent: number | null;
  predecessor: number | null;
};

export const run = async (invokeArgs: InvokeArgs): Promise<Payload> => {
  const { runId, deploymentPath, deploymentName } = invokeArgs;
  let { functionName, input, parent, predecessor, params } = invokeArgs;
  while (true) {
    const [id, response] = await invoke({
      runId,
      deploymentName,
      deploymentPath,
      functionName,
      input,
      parent,
      predecessor,
      params,
    });
    const next = response.next;
    if (!next) {
      return response.payload;
    } else if (typeof next === "string") {
      functionName = next;
      input = response.payload;
      params = null;
    } else {
      functionName = Object.keys(next)[0];
      input = response.payload;
      params = next[functionName] ?? null;
    }
    parent = null;
    predecessor = id;
  }
};

const invoke = async (
  invokeArgs: InvokeArgs,
): Promise<[number, ResponseBody]> => {
  const entry = findLookupEntry(invokeArgs.functionName);
  const invocationId = await createInvocation(invokeArgs);
  logEvent("DEBUG", "Invoking", {
    invocation: invocationId,
    function: invokeArgs.functionName,
    input: invokeArgs.input,
    params: invokeArgs.params,
  });

  let response: ResponseBody;
  if (entry.kind === "aws") {
    response = await invokeAwsFunction(entry, invokeArgs, invocationId);
  } else if (entry.kind === "kn") {
    response = await invokeKnFunction(entry, invokeArgs, invocationId);
  } else if (entry.kind === "plugin") {
    response = await invokePluginFunction(entry, invokeArgs, invocationId);
  } else {
    throw new Error(`Unknown function type: ${entry}`);
  }
  logEvent("INFO", "Completed", {
    invocation: invocationId,
    function: invokeArgs.functionName,
    input: invokeArgs.input,
    params: invokeArgs.params,
    output: response,
  });
  const endTime = new Date().toISOString();
  const output = JSON.stringify(response);
  using client = await globalThis.Pool.connect();
  await client.queryArray(
    "update invocations set output = $1, end_time = $2 where id = $3",
    [output, endTime, invocationId],
  );
  return [invocationId, response];
};

function findLookupEntry(functionName: string): LookupEntry {
  const entry = globalThis.LookupTable.find((f) => f.name === functionName);
  if (!entry) {
    throw new FunctionNotFoundError(functionName);
  }
  return entry;
}

async function createInvocation(invokeArgs: InvokeArgs) {
  const startTime = new Date().toISOString();
  const input = JSON.stringify(invokeArgs.input ?? null);
  const params = JSON.stringify(invokeArgs.params ?? null);
  using client = await globalThis.Pool.connect();
  const { rows } = await client.queryObject<{ id: number }>(
    "insert into invocations (parent, predecessor, run_id, function_name, input, params, start_time) values ($1, $2, $3, $4, $5, $6, $7) returning id",
    [
      invokeArgs.parent ?? null,
      invokeArgs.predecessor ?? null,
      invokeArgs.runId,
      invokeArgs.functionName,
      input,
      params,
      startTime,
    ],
  );
  return rows[0].id;
}
