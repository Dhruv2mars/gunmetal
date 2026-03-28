import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

import { NextResponse } from "next/server";

import {
  extractAssistantText,
  normalizeBaseUrl,
  roundMs,
  type PerfTransport,
} from "@/app/perf-lab/shared";

type RequestBody = {
  transport?: PerfTransport;
  apiKey?: string;
  baseUrl?: string;
  model?: string;
  prompt?: string;
  temperature?: number;
  maxTokens?: number;
};

type RpcNotification = {
  method: string;
  params: Record<string, unknown>;
};

type PendingResolver = {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
};

declare global {
  var __gunmetalPerfCodexClient: Promise<DirectCodexClient> | undefined;
}

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function POST(request: Request) {
  if (!isLocalHost(request.headers.get("host"))) {
    return NextResponse.json(
      {
        ok: false,
        status: 403,
        error: "Perf lab is local-only. Run it on localhost.",
        content: "",
        serverElapsedMs: 0,
      },
      { status: 403 },
    );
  }

  const body = (await request.json()) as RequestBody;
  const model = body.model?.trim() ?? "";
  const prompt = body.prompt?.trim() ?? "";
  const temperature = Number.isFinite(body.temperature) ? body.temperature : 0;
  const maxTokens = Number.isFinite(body.maxTokens) ? body.maxTokens : 8;

  if (!model || !prompt) {
    return NextResponse.json(
      {
        ok: false,
        status: 400,
        error: "Model and prompt are required.",
        content: "",
        serverElapsedMs: 0,
      },
      { status: 400 },
    );
  }

  const started = performance.now();

  try {
    if (body.transport === "codex-direct") {
      const client = await getCodexClient();
      const content = await client.runTurn({
        model,
        prompt,
      });

      return NextResponse.json({
        ok: true,
        status: 200,
        content,
        serverElapsedMs: roundMs(performance.now() - started),
      });
    }

    const apiKey = body.apiKey?.trim() ?? "";
    const baseUrl = normalizeBaseUrl(body.baseUrl ?? "");
    if (!apiKey || !baseUrl) {
      return NextResponse.json(
        {
          ok: false,
          status: 400,
          error: "API key and base URL are required for openai-compatible mode.",
          content: "",
          serverElapsedMs: roundMs(performance.now() - started),
        },
        { status: 400 },
      );
    }

    const upstream = await fetch(`${baseUrl}/chat/completions`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: prompt }],
        temperature,
        max_tokens: maxTokens,
        stream: false,
      }),
      cache: "no-store",
    });

    const text = await upstream.text();
    let content = text;
    try {
      content = extractAssistantText(JSON.parse(text)) || text;
    } catch {}

    return NextResponse.json(
      {
        ok: upstream.ok,
        status: upstream.status,
        content: content.trim(),
        error: upstream.ok ? undefined : text,
        serverElapsedMs: roundMs(performance.now() - started),
      },
      { status: upstream.ok ? 200 : upstream.status },
    );
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "Perf request failed.";
    return NextResponse.json(
      {
        ok: false,
        status: 500,
        error: message,
        content: "",
        serverElapsedMs: roundMs(performance.now() - started),
      },
      { status: 500 },
    );
  }
}

function isLocalHost(host: string | null): boolean {
  if (!host) {
    return false;
  }

  return /^(localhost|127\.0\.0\.1)(:\d+)?$/.test(host);
}

function codexBenchCwd(): string {
  const home = process.env.HOME;
  return home ? `${home}/.gunmetal/empty-workspace` : process.cwd();
}

async function getCodexClient(): Promise<DirectCodexClient> {
  globalThis.__gunmetalPerfCodexClient ??= DirectCodexClient.connect();
  return globalThis.__gunmetalPerfCodexClient;
}

class DirectCodexClient {
  private nextId = 1;
  private readonly pending = new Map<number, PendingResolver>();
  private readonly notifications: RpcNotification[] = [];
  private readonly waiters: Array<(notification: RpcNotification) => void> = [];

  private constructor(
    readonly child: ReturnType<typeof spawn>,
  ) {
    const reader = createInterface({ input: child.stdout! });
    reader.on("line", (line) => {
      if (!line.trim()) {
        return;
      }

      const message = JSON.parse(line) as
        | { id?: number; result?: unknown; error?: { message?: string }; method?: string; params?: Record<string, unknown> }
        | undefined;
      if (!message) {
        return;
      }

      if (message.method) {
        const notification = {
          method: message.method,
          params: message.params ?? {},
        };
        const waiter = this.waiters.shift();
        if (waiter) {
          waiter(notification);
        } else {
          this.notifications.push(notification);
        }
        return;
      }

      if (typeof message.id !== "number") {
        return;
      }

      const resolver = this.pending.get(message.id);
      if (!resolver) {
        return;
      }

      this.pending.delete(message.id);
      if (message.error) {
        resolver.reject(new Error(message.error.message ?? "Codex RPC error"));
      } else {
        resolver.resolve(message.result);
      }
    });

    child.once("exit", () => {
      globalThis.__gunmetalPerfCodexClient = undefined;
    });
  }

  static async connect(): Promise<DirectCodexClient> {
    const child = spawn("codex", ["app-server", "--listen", "stdio://"], {
      stdio: ["pipe", "pipe", "ignore"],
    });
    const client = new DirectCodexClient(child);
    await client.request("initialize", {
      clientInfo: { name: "gunmetal-perf-lab", title: "perf-lab", version: "1.0.0" },
    });
    client.notify("initialized", {});
    return client;
  }

  async runTurn(input: { model: string; prompt: string }): Promise<string> {
    const thread = (await this.request("thread/start", {
      approvalPolicy: "never",
      cwd: codexBenchCwd(),
      developerInstructions:
        "You are answering a normal conversational request. Do not perform file edits or tool actions.",
      model: input.model,
      personality: "friendly",
      sandbox: "read-only",
      serviceName: "gunmetal",
    })) as { thread: { id: string } };

    const turn = (await this.request("turn/start", {
      approvalPolicy: "never",
      cwd: codexBenchCwd(),
      input: [{ type: "text", text: `user: ${input.prompt}` }],
      model: input.model,
      personality: "friendly",
      sandboxPolicy: { type: "readOnly", networkAccess: false },
      summary: "concise",
      threadId: thread.thread.id,
    })) as { turn: { id: string } };

    let output = "";
    while (true) {
      const notification = await this.nextNotification();
      if (!matchesTurnNotification(notification, thread.thread.id, turn.turn.id)) {
        continue;
      }

      if (notification.method === "item/agentMessage/delta") {
        const delta = notification.params.delta;
        if (typeof delta === "string") {
          output += delta;
        }
      } else if (notification.method === "error") {
        const error = notification.params.error;
        throw new Error(
          typeof error === "object" && error && "message" in error
            ? String(error.message)
            : "Codex direct turn failed.",
        );
      } else if (notification.method === "turn/completed") {
        return output.trim();
      }
    }
  }

  private request(method: string, params: Record<string, unknown>): Promise<unknown> {
    const id = this.nextId;
    this.nextId += 1;
    this.child.stdin!.write(`${JSON.stringify({ id, method, params })}\n`);
    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });
  }

  private notify(method: string, params: Record<string, unknown>) {
    this.child.stdin!.write(`${JSON.stringify({ method, params })}\n`);
  }

  private nextNotification(): Promise<RpcNotification> {
    return this.notifications.length > 0
      ? Promise.resolve(this.notifications.shift()!)
      : new Promise<RpcNotification>((resolve) => {
          this.waiters.push(resolve);
        });
  }
}

function matchesTurnNotification(
  notification: RpcNotification,
  threadId: string,
  turnId: string,
) {
  const params = notification.params;
  const directThread =
    params.threadId === threadId ||
    ((params.turn as { threadId?: string } | undefined)?.threadId ?? "") ===
      threadId;

  if (notification.method === "thread/tokenUsage/updated") {
    return directThread;
  }

  return (
    directThread &&
    (params.turnId === turnId ||
      ((params.turn as { id?: string } | undefined)?.id ?? "") === turnId)
  );
}
