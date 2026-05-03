import { copyToClipboard } from "./utils";

const API_BASE = "/webui/api";

export interface ServiceInfo {
  status: string;
  version: string;
  home: string;
  api_base_url: string;
  web_url: string;
}

export interface Counts {
  profiles: number;
  models: number;
  keys: number;
  logs: number;
}

export interface SetupState {
  provider_ready: boolean;
  models_ready: boolean;
  key_ready: boolean;
  traffic_ready: boolean;
  next_step: string;
}

export interface Traffic {
  recent_requests: number;
  success_count: number;
  error_count: number;
  avg_latency_ms: number | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  latest_request_at: string | null;
}

export interface ProviderSummary {
  provider: string;
  profile_name: string | null;
  label: string;
  requests: number;
  success_count: number;
  error_count: number;
  avg_latency_ms: number | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export interface ModelSummary {
  model: string;
  provider: string;
  requests: number;
  success_count: number;
  error_count: number;
  avg_latency_ms: number | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export interface ProviderDef {
  kind: string;
  label: string;
  class: string;
  auth_method: string;
  supports_base_url: boolean;
  supports_model_sync: boolean;
  supports_chat_completions: boolean;
  supports_responses_api: boolean;
  supports_streaming: boolean;
  helper_title: string;
  helper_body: string;
  suggested_name: string;
  base_url_placeholder: string;
}

export interface Profile {
  id: string;
  provider: string;
  name: string;
  selector: string;
  base_url: string | null;
  auth_label: string;
  model_count: number;
}

export interface Model {
  id: string;
  provider: string;
  supports_reasoning: boolean | null;
  supports_tools: boolean | null;
}

export interface Key {
  id: string;
  name: string;
  prefix: string;
  state: string;
  scopes: string[];
  providers: string[];
  last_used_at: string | null;
}

export interface Log {
  id: string;
  started_at: string;
  provider: string;
  profile_name: string | null;
  model: string;
  endpoint: string;
  status_code: number | null;
  duration_ms: number;
  key_name: string | null;
  input_tokens: number | null;
  output_tokens: number | null;
  total_tokens: number | null;
  request_mode: string;
  error_message: string | null;
}

export interface AppState {
  service: ServiceInfo;
  counts: Counts;
  setup: SetupState;
  traffic: Traffic;
  provider_summaries: ProviderSummary[];
  model_summaries: ModelSummary[];
  providers: ProviderDef[];
  profiles: Profile[];
  models: Model[];
  keys: Key[];
  logs: Log[];
}

export interface ActionResponse {
  message: string;
  auth_url?: string;
  user_code?: string;
  secret?: string;
}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  const text = await response.text();
  const body = text ? JSON.parse(text) : {};
  if (!response.ok) {
    throw new Error(body?.error?.message || body?.message || "request failed");
  }
  return body;
}

export async function loadState(): Promise<AppState> {
  return fetchJson<AppState>(`${API_BASE}/state`);
}

export async function createProfile(payload: {
  provider: string;
  name: string;
  base_url?: string;
  api_key?: string;
}): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/profiles`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export async function profileAction(id: string, action: string): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/profiles/${id}/${action}`, { method: "POST" });
}

export async function createKey(profileId: string, name: string): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/profiles/${profileId}/keys`, {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export async function setKeyState(id: string, state: string): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/keys/${id}/state`, {
    method: "POST",
    body: JSON.stringify({ state }),
  });
}

export async function deleteKey(id: string): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/keys/${id}`, { method: "DELETE" });
}

export async function deleteProfile(id: string): Promise<ActionResponse> {
  return fetchJson<ActionResponse>(`${API_BASE}/profiles/${id}`, { method: "DELETE" });
}

export async function sendPlayground(
  mode: "chat" | "responses",
  key: string,
  model: string,
  messages: { role: string; content: string }[]
): Promise<Response> {
  const endpoint = mode === "responses" ? "/v1/responses" : "/v1/chat/completions";
  const payload =
    mode === "responses"
      ? {
          model,
          stream: true,
          input: messages.map((m) => ({
            role: m.role === "system" ? "developer" : m.role,
            content: [{ type: "input_text", text: m.content }],
          })),
        }
      : { model, stream: true, messages };

  return fetch(endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${key}`,
    },
    body: JSON.stringify(payload),
  });
}

export async function copyApiUrl(): Promise<boolean> {
  const base = new URL("/v1", window.location.origin).toString();
  return copyToClipboard(base);
}
