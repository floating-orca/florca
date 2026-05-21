import type {
  DeploymentName,
  InvocationId,
  LookupEntry,
  RunId,
} from "@florca/types";
import type { EventSink } from "./event_sink.ts";
import type { InvocationLoggerFactory } from "./invocation_logger.ts";
import type { WorkflowLogger } from "./workflow_logger.ts";

export type MessageHandler = (message: any) => any;

export type DriverState = {
  runId: RunId;
  deploymentName: DeploymentName;
  deploymentPath: string;
  lookupTable: LookupEntry[];
  messageHandlers: Map<InvocationId, MessageHandler>;
  workflowMessageHandler: MessageHandler | null;
  eventSink: EventSink;
  invocationLoggerFactory: InvocationLoggerFactory;
  workflowLogger: WorkflowLogger;
};
