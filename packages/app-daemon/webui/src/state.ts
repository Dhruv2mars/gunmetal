import type { AppState, Log } from "./api";

export interface PlaygroundState {
  key: string;
  provider: string;
  model: string;
  mode: "chat" | "responses";
  historyMode: "conversation" | "single";
  running: boolean;
  messages: { role: string; content: string }[];
  usage: { input_tokens?: number; output_tokens?: number; total_tokens?: number } | null;
  lastDurationMs: number | null;
}

export interface RequestFilters {
  provider: string;
  status: string;
  query: string;
  limit: string;
}

export interface UIState {
  data: AppState | null;
  loading: boolean;
  toast: { message: string; tone: "default" | "success" | "danger" } | null;
  secret: string | null;
  selectedLogId: string | null;
  expandedLogId: string | null;
  expandedSection: string | null;
  showAddProvider: boolean;
  showCreateKey: boolean;
  modelSearch: string;
  playground: PlaygroundState;
  requestFilters: RequestFilters;
}

export const initialPlayground: PlaygroundState = {
  key: "",
  provider: "",
  model: "",
  mode: "chat",
  historyMode: "conversation",
  running: false,
  messages: [],
  usage: null,
  lastDurationMs: null,
};

export const initialFilters: RequestFilters = {
  provider: "all",
  status: "all",
  query: "",
  limit: "8",
};

export const uiState: UIState = {
  data: null,
  loading: true,
  toast: null,
  secret: null,
  selectedLogId: null,
  expandedLogId: null,
  expandedSection: null,
  showAddProvider: false,
  showCreateKey: false,
  modelSearch: "",
  playground: { ...initialPlayground },
  requestFilters: { ...initialFilters },
};

let renderFn: (() => void) | null = null;

export function setRenderer(fn: () => void) {
  renderFn = fn;
}

export function render() {
  renderFn?.();
}

export function setToast(message: string, tone: "default" | "success" | "danger" = "default") {
  uiState.toast = { message, tone };
  render();
  setTimeout(() => {
    if (uiState.toast?.message === message) {
      uiState.toast = null;
      render();
    }
  }, 3000);
}

export function setSecret(secret: string | null) {
  uiState.secret = secret;
  if (secret) {
    uiState.playground.key = secret;
  }
  render();
}

export function providerFilterValue(provider: string, profileName: string | null): string {
  return `${provider}::${profileName || ""}`;
}

export function logMatchesFilters(log: Log): boolean {
  const f = uiState.requestFilters;
  const providerMatches =
    f.provider === "all" || providerFilterValue(log.provider, log.profile_name) === f.provider;
  const statusMatches =
    f.status === "all" ||
    (f.status === "success"
      ? (log.status_code ?? 0) < 400 && !log.error_message
      : (log.status_code ?? 0) >= 400 || Boolean(log.error_message));
  const query = f.query.trim().toLowerCase();
  const queryMatches =
    !query ||
    [
      log.provider,
      log.profile_name || "",
      log.model,
      log.endpoint,
      log.key_name || "",
      log.request_mode || "",
      log.error_message || "",
    ]
      .join(" ")
      .toLowerCase()
      .includes(query);
  return providerMatches && statusMatches && queryMatches;
}
