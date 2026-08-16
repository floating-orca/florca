import { expect } from "@std/expect";
import { EventBatcher } from "../lib/event_batcher.ts";
import type { DriverEvent } from "@florca/types";

Deno.env.set("ENGINE_URL", "http://engine.invalid:8080");
Deno.env.set("BASIC_AUTH_USERNAME", "user");
Deno.env.set("BASIC_AUTH_PASSWORD", "password");

const NEVER_MS = 100_000;

function logEvent(message: string): DriverEvent {
  return {
    type: "log",
    scope: "workflow",
    level: "INFO",
    message,
    data: null,
  };
}

interface SentRequest {
  url: string;
  authorization: string | null;
  events: DriverEvent[];
}

function stubFetch(
  requests: SentRequest[],
  status: () => number = () => 200,
): () => void {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (input: URL | RequestInfo, init?: RequestInit) => {
    requests.push({
      url: String(input),
      authorization: new Headers(init?.headers).get("Authorization"),
      events: JSON.parse(String(init?.body)) as DriverEvent[],
    });
    return Promise.resolve(new Response(null, { status: status() }));
  };
  return () => {
    globalThis.fetch = originalFetch;
  };
}

function stubConsoleError(captured?: string[]): () => void {
  const originalConsoleError = console.error;
  console.error = (...args: unknown[]) => {
    captured?.push(String(args[0]));
  };
  return () => {
    console.error = originalConsoleError;
  };
}

function messagesOf(request: SentRequest): string[] {
  return request.events.map((event) => (event as { message: string }).message);
}

Deno.test("EventBatcher flushes when the batch reaches its size threshold", async () => {
  const requests: SentRequest[] = [];
  const restoreFetch = stubFetch(requests);
  try {
    const batcher = new EventBatcher(7, {
      maxBatchSize: 3,
      flushIntervalMs: NEVER_MS,
    });
    batcher.addEvent(logEvent("one"));
    batcher.addEvent(logEvent("two"));
    expect(requests).toHaveLength(0);
    batcher.addEvent(logEvent("three"));
    await batcher.flush();

    expect(requests).toHaveLength(1);
    expect(requests[0].url).toBe("http://engine.invalid:8080/7/events");
    expect(requests[0].authorization).toBe(`Basic ${btoa("user:password")}`);
    expect(messagesOf(requests[0])).toEqual(["one", "two", "three"]);
  } finally {
    restoreFetch();
  }
});

Deno.test("EventBatcher flushes on the timer below the size threshold", async () => {
  const requests: SentRequest[] = [];
  const { promise: timerSend, resolve: resolveTimerSend } = Promise
    .withResolvers<void>();
  const restoreFetch = stubFetch(requests, () => {
    resolveTimerSend();
    return 200;
  });
  try {
    const batcher = new EventBatcher(7, { flushIntervalMs: 10 });
    batcher.addEvent(logEvent("one"));
    batcher.addEvent(logEvent("two"));
    await timerSend;
    await batcher.flush();

    expect(requests).toHaveLength(1);
    expect(messagesOf(requests[0])).toEqual(["one", "two"]);
  } finally {
    restoreFetch();
  }
});

Deno.test("EventBatcher preserves events after a failed send and resends them in order", async () => {
  const requests: SentRequest[] = [];
  let nextStatus = 500;
  const restoreFetch = stubFetch(requests, () => nextStatus);
  const restoreConsoleError = stubConsoleError();
  try {
    const batcher = new EventBatcher(7, { flushIntervalMs: NEVER_MS });
    batcher.addEvent(logEvent("one"));
    batcher.addEvent(logEvent("two"));
    await batcher.flush();
    expect(requests).toHaveLength(1);

    nextStatus = 200;
    batcher.addEvent(logEvent("three"));
    await batcher.flush();

    expect(requests).toHaveLength(2);
    expect(messagesOf(requests[1])).toEqual(["one", "two", "three"]);
  } finally {
    restoreConsoleError();
    restoreFetch();
  }
});

Deno.test("finalFlush retries failed sends until the events are delivered", async () => {
  const requests: SentRequest[] = [];
  let failures = 2;
  const restoreFetch = stubFetch(requests, () => (failures-- > 0 ? 500 : 200));
  const restoreConsoleError = stubConsoleError();
  try {
    const batcher = new EventBatcher(7, {
      flushIntervalMs: NEVER_MS,
      finalFlushDelaysMs: [1, 1, 1],
    });
    batcher.addEvent(logEvent("one"));
    batcher.addEvent(logEvent("two"));
    await batcher.finalFlush();

    expect(requests).toHaveLength(3);
    expect(messagesOf(requests[2])).toEqual(["one", "two"]);
  } finally {
    restoreConsoleError();
    restoreFetch();
  }
});

Deno.test("finalFlush gives up after its retries and reports the loss", async () => {
  const requests: SentRequest[] = [];
  const restoreFetch = stubFetch(requests, () => 500);
  const errors: string[] = [];
  const restoreConsoleError = stubConsoleError(errors);
  try {
    const batcher = new EventBatcher(7, {
      flushIntervalMs: NEVER_MS,
      finalFlushDelaysMs: [1, 1],
    });
    batcher.addEvent(logEvent("one"));
    await batcher.finalFlush();

    expect(requests).toHaveLength(3);
    expect(errors.some((e) => e.includes("could not be delivered"))).toBe(true);
  } finally {
    restoreConsoleError();
    restoreFetch();
  }
});

Deno.test("EventBatcher sends an oversized batch in chunks, keeping order", async () => {
  const requests: SentRequest[] = [];
  const restoreFetch = stubFetch(requests);
  try {
    const batcher = new EventBatcher(7, {
      maxBatchSize: 2,
      flushIntervalMs: NEVER_MS,
    });
    for (const message of ["one", "two", "three", "four", "five"]) {
      batcher.addEvent(logEvent(message));
    }
    await batcher.flush();

    expect(requests.map(messagesOf)).toEqual([
      ["one", "two"],
      ["three", "four"],
      ["five"],
    ]);
  } finally {
    restoreFetch();
  }
});
