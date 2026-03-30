import type {
  DriverArgs,
  DriverResult,
  LogEvent,
  LogLevel,
  LookupEntry,
  ReportReadinessRequest,
  RunId,
} from "@florca/types";
import { flushWriteQueue, run } from "./run.ts";
import { resolve } from "@std/path";
import { getPluginFilePath, namesOfShippedPlugins } from "./functions/mod.ts";

export function logEvent(level: LogLevel, message: string, data?: any) {
  const logEvent: LogEvent = {
    level,
    message,
    data,
  };
  console.log(JSON.stringify(logEvent));
}

export const AUTHORIZATION_HEADER = `Basic ${
  btoa(
    Deno.env.get("BASIC_AUTH_USERNAME") +
      ":" +
      Deno.env.get("BASIC_AUTH_PASSWORD"),
  )
}`;

export async function gatherLookupEntries(
  deploymentPath: string,
): Promise<LookupEntry[]> {
  let lookupFunctions: LookupEntry[] = [];

  const workflowPlugins = JSON.parse(
    await Deno.readTextFile(resolve(deploymentPath, "lookup.json")),
  ) as LookupEntry[];
  lookupFunctions = lookupFunctions.concat(workflowPlugins);

  const shippedPlugins = (await namesOfShippedPlugins()).map(
    (name): LookupEntry => ({
      kind: "plugin",
      name,
      location: getPluginFilePath(name),
    }),
  );
  lookupFunctions = lookupFunctions.concat(shippedPlugins);

  return lookupFunctions;
}

export function getEngineUrl(): string {
  const url = Deno.env.get("ENGINE_URL");
  if (!url) {
    throw new Error("No ENGINE_URL environment variable set");
  }
  return url;
}

export function getEngineUrlForAccessFromKn(): string {
  const url = Deno.env.get("ENGINE_URL_FOR_ACCESS_FROM_KN");
  if (!url) {
    throw new Error(
      "No ENGINE_URL_FOR_ACCESS_FROM_KN environment variable set",
    );
  }
  return url;
}

export async function reportAvailabilityToEngine(runId: RunId, port: number) {
  const reportReadinessRequest: ReportReadinessRequest = {
    port: port,
    runId: runId,
  };
  await fetch(`${getEngineUrl()}/ready`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: AUTHORIZATION_HEADER,
    },
    body: JSON.stringify(reportReadinessRequest),
  });
}

export async function runWorkflow(
  driverArgs: DriverArgs,
): Promise<DriverResult> {
  let driverResult: DriverResult;
  try {
    const result = await run({
      ...driverArgs,
      functionName: driverArgs.entryPoint,
      parent: null,
      predecessor: null,
    });
    driverResult = {
      runId: driverArgs.runId,
      result: {
        success: {
          value: result,
        },
      },
    };
  } catch (e) {
    if (e instanceof Error) {
      driverResult = {
        runId: driverArgs.runId,
        result: {
          error: {
            kind: e.constructor.name,
            message: e.message,
          },
        },
      };
    } else {
      throw e;
    }
  } finally {
    await flushWriteQueue();
  }
  return driverResult;
}
