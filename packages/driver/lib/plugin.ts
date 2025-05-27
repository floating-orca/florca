// deno-lint-ignore-file no-explicit-any

import "@std/dotenv/load";
import { resolve } from "@std/path";
import type {
  LogLevel,
  Payload,
  PluginRequestBody,
  ResponseBody,
} from "@florca/fn";
import type { InvocationId, LookupEntry, PluginLogEvent } from "@florca/types";
import { type InvokeArgs, run } from "./run.ts";
import { AUTHORIZATION_HEADER, getEngineUrl } from "./mod.ts";

export async function invokePluginFunction(
  entry: LookupEntry,
  invokeArgs: InvokeArgs,
  invocationId: InvocationId,
): Promise<ResponseBody> {
  const plugin = await import(
    resolve(invokeArgs.deploymentPath, entry.location)
  );
  const body: PluginRequestBody = {
    payload: invokeArgs.input,
    context: {
      authorizationHeader: AUTHORIZATION_HEADER,
      id: invocationId,
      params: invokeArgs.params,
      parentId: invokeArgs.parent,
      workflowMessageUrl: `${getEngineUrl()}/${invokeArgs.runId}`,
      logEvent: (level: LogLevel, message: string, data?: any) => {
        const pluginLogEvent: PluginLogEvent = {
          level,
          message,
          data,
          invocationId: invocationId,
          functionName: invokeArgs.functionName,
        };
        console.log(JSON.stringify(pluginLogEvent));
      },
      onMessage: (fn: ((message: any) => void) | null) => {
        if (fn) {
          globalThis.MessageHandlers.set(invocationId, fn);
        } else {
          globalThis.MessageHandlers.delete(invocationId);
        }
      },
      onWorkflowMessage: (fn: ((message: any) => void) | null) => {
        globalThis.WorkflowMessageHandler = fn;
      },
      run: (fn: string | any, payload: Payload) => {
        let functionName;
        let params;
        if (typeof fn === "string") {
          functionName = fn;
        } else {
          functionName = Object.keys(fn)[0];
          params = fn[functionName];
        }
        const runArgs: InvokeArgs = {
          runId: invokeArgs.runId,
          deploymentName: invokeArgs.deploymentName,
          deploymentPath: invokeArgs.deploymentPath,
          functionName,
          input: payload,
          params: params ?? null,
          parent: invocationId,
          predecessor: null,
        };
        return run(runArgs);
      },
    },
  };
  const response = await plugin.default(body);
  return response;
}
