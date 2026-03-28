export type PerfTransport = "openai-compatible" | "codex-direct";

export type PerfPreset = {
  id: string;
  label: string;
  provider: "openrouter" | "zen" | "codex";
  route: "upstream" | "gunmetal";
  transport: PerfTransport;
  model: string;
  baseUrl: string;
  prompt: string;
  temperature: number;
  maxTokens: number;
  note: string;
};

export type RunSummary = {
  minMs: number;
  medianMs: number;
  maxMs: number;
};

export const DEFAULT_PROMPT = "Reply with exactly OK.";

export const perfPresets: PerfPreset[] = [
  {
    id: "openrouter-upstream",
    label: "OpenRouter Upstream",
    provider: "openrouter",
    route: "upstream",
    transport: "openai-compatible",
    model: "nvidia/nemotron-3-nano-30b-a3b:free",
    baseUrl: "https://openrouter.ai/api/v1",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Direct OpenRouter path with the free Nemotron Nano model.",
  },
  {
    id: "openrouter-gunmetal",
    label: "OpenRouter via Gunmetal",
    provider: "openrouter",
    route: "gunmetal",
    transport: "openai-compatible",
    model: "openrouter/nvidia/nemotron-3-nano-30b-a3b:free",
    baseUrl: "http://127.0.0.1:4684/v1",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Same model through Gunmetal's local OpenAI-compatible surface.",
  },
  {
    id: "zen-upstream",
    label: "Zen Upstream",
    provider: "zen",
    route: "upstream",
    transport: "openai-compatible",
    model: "mimo-v2-flash-free",
    baseUrl: "https://opencode.ai/zen/v1",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Direct Zen path with MiMo V2 Flash Free.",
  },
  {
    id: "zen-gunmetal",
    label: "Zen via Gunmetal",
    provider: "zen",
    route: "gunmetal",
    transport: "openai-compatible",
    model: "zen/mimo-v2-flash-free",
    baseUrl: "http://127.0.0.1:4684/v1",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Same Zen model through Gunmetal.",
  },
  {
    id: "codex-upstream",
    label: "Codex Direct",
    provider: "codex",
    route: "upstream",
    transport: "codex-direct",
    model: "gpt-5.4-mini",
    baseUrl: "",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Direct local Codex app-server path. No API key or base URL field needed.",
  },
  {
    id: "codex-gunmetal",
    label: "Codex via Gunmetal",
    provider: "codex",
    route: "gunmetal",
    transport: "openai-compatible",
    model: "codex/gpt-5.4-mini",
    baseUrl: "http://127.0.0.1:4684/v1",
    prompt: DEFAULT_PROMPT,
    temperature: 0,
    maxTokens: 8,
    note: "Same Codex model through Gunmetal's local HTTP surface.",
  },
];

export function extractAssistantText(payload: unknown): string {
  if (!payload || typeof payload !== "object") {
    return "";
  }

  const choices = Reflect.get(payload, "choices");
  if (!Array.isArray(choices) || choices.length === 0) {
    return "";
  }

  const message = choices[0];
  if (!message || typeof message !== "object") {
    return "";
  }

  const content = Reflect.get(Reflect.get(message, "message") ?? {}, "content");
  if (typeof content === "string") {
    return content.trim();
  }

  if (!Array.isArray(content)) {
    return "";
  }

  return content
    .map((part) => {
      if (!part || typeof part !== "object") {
        return "";
      }
      const text = Reflect.get(part, "text");
      return typeof text === "string" ? text : "";
    })
    .join("")
    .trim();
}

export function median(values: number[]): number {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0
    ? (sorted[middle - 1] + sorted[middle]) / 2
    : sorted[middle];
}

export function roundMs(value: number): number {
  return Math.round(value * 100) / 100;
}

export function summarizeRuns(values: number[]): RunSummary | null {
  if (values.length === 0) {
    return null;
  }

  return {
    minMs: roundMs(Math.min(...values)),
    medianMs: roundMs(median(values)),
    maxMs: roundMs(Math.max(...values)),
  };
}

export function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.trim().replace(/\/+$/, "");
}
