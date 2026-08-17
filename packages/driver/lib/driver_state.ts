import type {
  DeploymentName,
  InvocationId,
  LookupEntry,
  RunId,
} from "@florca/types";
import type { EventSink } from "./event_sink.ts";
import type { InvocationLoggerFactory } from "./invocation_logger.ts";
import type { InvokeArgs } from "./invoke_args.ts";
import type { WorkflowLogger } from "./workflow_logger.ts";
import type { LambdaClient } from "@aws-sdk/client-lambda";

export type MessageHandler = (message: any) => any;

export type InFlightInvocation = {
  args: InvokeArgs;
  startTime: Temporal.Instant;
};

export type DriverState = {
  runId: RunId;
  deploymentName: DeploymentName;
  deploymentPath: string;
  lookupTable: LookupEntry[];
  messageHandlers: Map<InvocationId, MessageHandler>;
  workflowMessageHandler: MessageHandler | null;
  inFlightInvocations: Map<InvocationId, InFlightInvocation>;
  eventSink: EventSink;
  invocationLoggerFactory: InvocationLoggerFactory;
  workflowLogger: WorkflowLogger;
  lambdaClients: Map<string, LambdaClient>;
};
