import type { InvocationId, LookupEntry } from "@florca/types";
import type { Pool } from "@db/postgres";

// deno-lint-ignore no-explicit-any
export type MessageHandler = (message: any) => any;

export type DriverState = {
  lookupTable: LookupEntry[];
  messageHandlers: Map<InvocationId, MessageHandler>;
  workflowMessageHandler: MessageHandler | null;
  pool: Pool;
};
