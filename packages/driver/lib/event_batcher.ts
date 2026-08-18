import type { DriverEvent, RunId } from "@florca/types";
import { getAuthorizationHeader } from "./auth.ts";
import * as env from "./env.ts";
import type { EventSink } from "./event_sink.ts";

export class EventBatcher implements EventSink {
  private readonly runId: RunId;
  private readonly maxBatchSize: number;
  private readonly flushIntervalMs: number;
  private readonly finalFlushDelaysMs: number[];

  private batch: DriverEvent[] = [];
  private flushTimer: ReturnType<typeof setTimeout> | null = null;
  private flushChain: Promise<void> = Promise.resolve();

  constructor(
    runId: RunId,
    options: {
      maxBatchSize?: number;
      flushIntervalMs?: number;
      finalFlushDelaysMs?: number[];
    } = {},
  ) {
    this.runId = runId;
    this.maxBatchSize = options.maxBatchSize ?? 100;
    this.flushIntervalMs = options.flushIntervalMs ?? 100;
    this.finalFlushDelaysMs = options.finalFlushDelaysMs ??
      [250, 500, 1000, 2000];
  }

  addEvent(event: DriverEvent): void {
    this.batch.push(event);

    if (this.batch.length >= this.maxBatchSize) {
      void this.flush();
    } else if (this.flushTimer === null) {
      this.flushTimer = setTimeout(() => {
        void this.flush();
      }, this.flushIntervalMs);
    }
  }

  async flush(): Promise<void> {
    this.flushChain = this.flushChain.then(async () => {
      if (this.batch.length === 0) return;

      this.clearFlushTimer();

      const batch = this.batch;
      this.batch = [];

      await this.sendEventChunks(batch);
    });

    await this.flushChain;
  }

  // Failed sends have no later delivery to wait for here, so retry with
  // backoff and report what could not be delivered
  async finalFlush(): Promise<void> {
    await this.flush();
    for (const delayMs of this.finalFlushDelaysMs) {
      if (this.batch.length === 0) {
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      await this.flush();
    }
    if (this.batch.length > 0) {
      console.error(
        `${this.batch.length} event(s) could not be delivered, the inspection of run ${this.runId} is incomplete.`,
      );
    }
  }

  private clearFlushTimer(): void {
    if (this.flushTimer !== null) {
      clearTimeout(this.flushTimer);
      this.flushTimer = null;
    }
  }

  private async sendEventChunks(events: DriverEvent[]): Promise<void> {
    let nextIndex = 0;
    try {
      const url = `${env.getEngineUrl()}/${this.runId}/events`;
      while (nextIndex < events.length) {
        const eventChunk = events.slice(
          nextIndex,
          nextIndex + this.maxBatchSize,
        );
        const response = await fetch(url, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: getAuthorizationHeader(),
          },
          body: JSON.stringify(eventChunk),
          // An unresponsive engine must not block flushes (or the SIGTERM handler) forever
          signal: AbortSignal.timeout(10_000),
        });

        if (!response.ok) {
          throw await this.newHttpRequestError(response, url);
        }

        nextIndex += eventChunk.length;
      }
    } catch (error) {
      const unsentEvents = events.slice(nextIndex);
      this.batch = [...unsentEvents, ...this.batch];

      console.error(
        `Failed to send event batch for run ${this.runId}; sent ${nextIndex}/${events.length}, preserved ${unsentEvents.length} event(s) for later delivery.`,
        error,
      );
    }
  }

  private async newHttpRequestError(
    response: Response,
    url: string,
  ): Promise<Error> {
    const errorText = await response.text().catch(() => "");
    return new Error(
      `HTTP request to ${url} failed: ${response.status} ${response.statusText}\n${errorText}`,
    );
  }
}
