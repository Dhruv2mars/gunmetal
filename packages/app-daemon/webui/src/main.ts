import "./styles.css";

/* -------------------------------------------------------------------------- */
/*                                 Types                                      */
/* -------------------------------------------------------------------------- */

interface SetupState {
  provider_ready: boolean;
  models_ready: boolean;
  key_ready: boolean;
  traffic_ready: boolean;
}

interface ServiceInfo {
  api_base_url: string;
  version: string;
  home: string;
}

interface Counts {
  profiles: number;
  models: number;
  keys: number;
  logs: number;
}

interface Profile {
  id: string;
  name: string;
  provider: string;
  selector: string;
  model_count: number;
}

interface ProviderType {
  kind: string;
}

interface ModelInfo {
  id: string;
  provider: string;
  supports_reasoning?: boolean;
  supports_tools?: boolean;
}

interface KeyInfo {
  id: string;
  name: string;
  prefix: string;
  providers: string[];
  state: "active" | "disabled" | "revoked";
}

interface ProviderSummary {
  provider: string;
  profile_name?: string;
  label: string;
}

interface LogEntry {
  provider: string;
  profile_name?: string;
  model: string;
  endpoint: string;
  key_name?: string;
  request_mode?: string;
  status_code?: number;
  error_message?: string;
  started_at: string;
  duration_ms: number;
  total_tokens?: number;
}

interface AppState {
  setup: SetupState;
  service: ServiceInfo;
  counts: Counts;
  profiles: Profile[];
  providers: ProviderType[];
  models: ModelInfo[];
  keys: KeyInfo[];
  logs: LogEntry[];
  provider_summaries: ProviderSummary[];
}

/* -------------------------------------------------------------------------- */
/*                                 State                                      */
/* -------------------------------------------------------------------------- */

let appState: AppState | null = null;
let currentView = "overview";
let toastTimer: ReturnType<typeof setTimeout> | null = null;
let toast: { msg: string; ok: boolean } | null = null;

let modalOpen = false;
let modalType: "confirm" | "auth" | "create-key" | "delete-key" = "confirm";
let modalTitle = "";
let modalMessage = "";
let modalOnConfirm: (() => void) | null = null;
let modalProfileId = "";
let modalProviderKind = "";

let modelSearch = "";
let activityFilters = {
  provider: "all",
  status: "all",
  limit: "8",
  query: "",
};
let pgState = {
  provider: "",
  model: "",
  key: "",
  mode: "chat" as "chat" | "responses",
  messages: [] as { role: "user" | "assistant" | "system"; content: string }[],
  running: false,
  usage: null as null | { total_tokens?: number; prompt_tokens?: number; completion_tokens?: number },
};

/* -------------------------------------------------------------------------- */
/*                                 API                                        */
/* -------------------------------------------------------------------------- */

const API_BASE = "/webui/api";

async function apiFetch<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  const text = await res.text();
  const json = text ? (JSON.parse(text) as Record<string, unknown>) : {};
  if (!res.ok) {
    throw new Error(
      (json.error as { message?: string } | undefined)?.message ||
        (json.message as string | undefined) ||
        `HTTP ${res.status}`,
    );
  }
  return json as T;
}

const api = {
  state: () => apiFetch<AppState>(`${API_BASE}/state`),
  createProfile: (body: Record<string, string>) =>
    apiFetch<Record<string, unknown>>(`${API_BASE}/profiles`, {
      method: "POST",
      body: JSON.stringify(body),
    }),
  action: (id: string, action: string) =>
    apiFetch<Record<string, unknown>>(`${API_BASE}/profiles/${id}/${action}`, {
      method: "POST",
    }),
  createKey: (profileId: string, name: string) =>
    apiFetch<{ message: string; secret?: string }>(
      `${API_BASE}/profiles/${profileId}/keys`,
      { method: "POST", body: JSON.stringify({ name }) },
    ),
  setKeyState: (id: string, state: string) =>
    apiFetch<{ message: string }>(`${API_BASE}/keys/${id}/state`, {
      method: "POST",
      body: JSON.stringify({ state }),
    }),
  deleteKey: (id: string) =>
    apiFetch<{ message: string }>(`${API_BASE}/keys/${id}`, {
      method: "DELETE",
    }),
  deleteProfile: (id: string) =>
    apiFetch<{ message: string }>(`${API_BASE}/profiles/${id}`, {
      method: "DELETE",
    }),
  chat: (
    mode: "chat" | "responses",
    key: string,
    model: string,
    messages: { role: string; content: string }[],
  ) =>
    fetch(mode === "responses" ? "/v1/responses" : "/v1/chat/completions", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${key}`,
      },
      body: JSON.stringify(
        mode === "responses"
          ? {
              model,
              stream: true,
              input: messages.map((m) => ({
                role: m.role === "system" ? "developer" : m.role,
                content: [{ type: "input_text", text: m.content }],
              })),
            }
          : { model, stream: true, messages },
      ),
    }),
};

/* -------------------------------------------------------------------------- */
/*                                 Utils                                      */
/* -------------------------------------------------------------------------- */

const $ = (id: string) => document.getElementById(id);
const esc = (s: string | undefined | null) =>
  String(s ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

function fmtDate(d: string) {
  const date = new Date(d);
  return isNaN(date.getTime()) ? d : date.toLocaleString();
}

function fmtRelative(d: string) {
  const date = new Date(d);
  if (isNaN(date.getTime())) return d;
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}

function showToast(msg: string, ok = true) {
  toast = { msg, ok };
  renderToast();
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast = null;
    renderToast();
  }, 3000);
}

async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

/* Modal system */
function openConfirm(title: string, message: string, onConfirm: () => void) {
  modalOpen = true;
  modalType = "confirm";
  modalTitle = title;
  modalMessage = message;
  modalOnConfirm = onConfirm;
  renderModal();
}

function openAuthModal(providerKind: string) {
  modalOpen = true;
  modalType = "auth";
  modalTitle = `Connect ${providerKind}`;
  modalMessage = "";
  modalProviderKind = providerKind;
  renderModal();
}

function openCreateKeyModal() {
  modalOpen = true;
  modalType = "create-key";
  modalTitle = "Create API Key";
  modalMessage = "";
  renderModal();
}

function openDeleteKeyModal(keyId: string, keyName: string) {
  modalOpen = true;
  modalType = "delete-key";
  modalTitle = "Delete API Key";
  modalMessage = `Are you sure you want to delete "${keyName}"? This action cannot be undone.`;
  modalProfileId = keyId;
  modalOnConfirm = () => {
    api
      .deleteKey(keyId)
      .then((res) => {
        showToast(res.message);
        loadState();
      })
      .catch((e) => showToast((e as Error).message, false));
  };
  renderModal();
}

function closeModal() {
  modalOpen = false;
  modalOnConfirm = null;
  renderModal();
}

/* -------------------------------------------------------------------------- */
/*                                 Icons                                      */
/* -------------------------------------------------------------------------- */

const LogoIcon = () =>
  `<svg viewBox="0 0 120 120" fill="none" class="w-full h-full"><defs><clipPath id="bl"><rect x="0" y="0" width="60" height="120"/></clipPath></defs><circle cx="60" cy="60" r="52" stroke="currentColor" stroke-width="3" opacity="0.9"/><g clip-path="url(#bl)"><path d="M60 14 C58 16 56 18 54 21 C52 24 50 28 48 32 C44 40 42 48 41 56 C40 64 40 72 41 80 C42 84 43 88 45 90 C47 92 49 93 52 93 C54 93 56 92 58 90 C60 88 61 84 61 80 C61 72 61 64 61 56 C61 48 61 40 61 32 C61 28 61 24 61 21 C61 18 61 16 60 14Z" fill="currentColor" opacity="0.85"/></g><g transform="translate(120 0) scale(-1 1)" clip-path="url(#bl)"><path d="M60 14 C58 16 56 18 54 21 C52 24 50 28 48 32 C44 40 42 48 41 56 C40 64 40 72 41 80 C42 84 43 88 45 90 C47 92 49 93 52 93 C54 93 56 92 58 90 C60 88 61 84 61 80 C61 72 61 64 61 56 C61 48 61 40 61 32 C61 28 61 24 61 21 C61 18 61 16 60 14Z" fill="currentColor" opacity="0.35"/></g></svg>`;

const Icon = {
  overview:
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/></svg>',
  providers:
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 16V7a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9m16 0H4m16 0 1.28 2.55a1 1 0 0 1-.9 1.45H3.62a1 1 0 0 1-.9-1.45L4 16"/></svg>',
  models:
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
  keys: '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>',
  activity:
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>',
  playground:
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
  check:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>',
  search:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>',
  refresh:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>',
  copy: '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>',
  trash:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>',
  plus: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>',
  external:
    '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>',
  chevronRight:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>',
  info: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
  close:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>',
  warning:
    '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
  send: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>',
  circle:
    '<svg width="8" height="8" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="10"/></svg>',
  sparklineUp:
    '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/><polyline points="17 6 23 6 23 12"/></svg>',
  clock:
    '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
};

/* -------------------------------------------------------------------------- */
/*                                Rendering                                   */
/* -------------------------------------------------------------------------- */

function renderApp() {
  const app = $("app");
  if (!app) return;
  app.innerHTML = `
    <div class="flex h-dvh w-full overflow-hidden bg-bg">
      <aside class="w-[240px] shrink-0 border-r border-line flex flex-col z-20 bg-sidebar">
        <div class="h-14 shrink-0 flex items-center px-4 border-b border-line">
          <div class="flex items-center gap-2.5">
            <div class="h-[18px] w-[18px] text-text">${LogoIcon()}</div>
            <span class="text-sm font-medium tracking-tight">Gunmetal</span>
          </div>
        </div>
        <nav id="sidebar-nav" class="flex-1 overflow-y-auto p-2 space-y-0.5"></nav>
        <div id="sidebar-footer" class="p-3 border-t border-line shrink-0"></div>
      </aside>
      <main id="main-scroll" class="flex-1 overflow-y-auto relative bg-bg">
        <div id="main-content" class="max-w-5xl mx-auto p-8 lg:p-10"></div>
      </main>
      <div id="toast-container" class="fixed bottom-5 right-5 z-[100]"></div>
      <div id="modal-container" class="fixed inset-0 z-[60] pointer-events-none"></div>
    </div>
  `;
}

function renderSidebar() {
  const nav = $("sidebar-nav");
  const footer = $("sidebar-footer");
  if (!nav || !footer) return;

  if (!appState) {
    nav.innerHTML = `<div class="p-4 text-xs text-muted text-center animate-pulse">Loading...</div>`;
    return;
  }

  const items = [
    { id: "overview", label: "Overview", icon: Icon.overview },
    { id: "providers", label: "Providers", icon: Icon.providers, count: appState.counts.profiles },
    { id: "models", label: "Models", icon: Icon.models, count: appState.counts.models },
    { id: "keys", label: "API Keys", icon: Icon.keys, count: appState.counts.keys },
    { id: "activity", label: "Activity", icon: Icon.activity, count: appState.counts.logs },
    { id: "playground", label: "Playground", icon: Icon.playground },
  ];

  nav.innerHTML = items
    .map((item) => {
      const active = currentView === item.id;
      const activeClass = active
        ? "bg-surface text-text"
        : "text-muted hover:bg-surface hover:text-text";
      return `
        <button data-nav="${item.id}" class="w-full flex items-center justify-between px-3 py-[7px] rounded-lg text-[13px] interactive ${activeClass}">
          <div class="flex items-center gap-2.5">
            <span class="opacity-80">${item.icon}</span>
            <span>${item.label}</span>
          </div>
          ${item.count !== undefined && item.count !== null ? `<span class="text-[11px] font-mono opacity-40">${item.count}</span>` : ""}
        </button>
      `;
    })
    .join("");

  const apiUrl = new URL(appState.service.api_base_url, location.origin).toString();
  const statusColor = appState.setup.traffic_ready
    ? "#7dbeb3"
    : appState.setup.provider_ready
      ? "var(--color-accent)"
      : "#ff8d78";

  footer.innerHTML = `
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="h-2 w-2 rounded-full" style="background:${statusColor}"></span>
          <span class="text-[11px] text-muted">${appState.setup.traffic_ready ? "Online" : appState.setup.provider_ready ? "Setup" : "Idle"}</span>
        </div>
        <button id="btn-refresh" class="text-muted hover:text-text interactive p-1 rounded" title="Refresh">
          ${Icon.refresh}
        </button>
      </div>
      <div class="text-[11px] font-mono text-muted truncate" title="${esc(apiUrl)}">${esc(apiUrl)}</div>
      <button id="btn-copy-url" class="w-full mt-0.5 flex items-center justify-center gap-1.5 rounded-md border border-line px-3 py-[5px] text-[11px] interactive bg-surface text-text hover:border-line-hover">
        ${Icon.copy}
        Copy API URL
      </button>
      <div class="flex items-center justify-between text-[10px] text-faint mt-1">
        <span>v${esc(appState.service.version)}</span>
        <a href="${esc(appState.service.home)}" target="_blank" rel="noopener" class="hover:text-muted flex items-center gap-1">Docs ${Icon.external}</a>
      </div>
    </div>
  `;
}

function renderToast() {
  const container = $("toast-container");
  if (!container) return;
  if (!toast) {
    container.innerHTML = "";
    return;
  }
  const color = toast.ok ? "#7dbeb3" : "#ff8d78";
  const bg = toast.ok ? "rgba(125,211,200,0.06)" : "rgba(255,141,120,0.06)";
  const border = toast.ok ? "rgba(125,211,200,0.2)" : "rgba(255,141,120,0.2)";
  container.innerHTML = `
    <div class="flex items-center gap-2.5 rounded-xl border px-4 py-2.5 shadow-2xl" style="background:${bg};border-color:${border};backdrop-filter:blur(12px);animation:toastIn 200ms var(--ease-out) both">
      <span class="h-2 w-2 rounded-full shrink-0" style="background:${color}"></span>
      <span class="text-[13px] text-text">${esc(toast.msg)}</span>
    </div>
  `;
}

function renderModal() {
  const container = $("modal-container");
  if (!container) return;
  if (!modalOpen) {
    container.innerHTML = "";
    container.classList.add("pointer-events-none");
    return;
  }
  container.classList.remove("pointer-events-none");

  let body = "";
  if (modalType === "confirm" || modalType === "delete-key") {
    body = `
      <p class="text-[13px] text-muted leading-relaxed mt-1">${esc(modalMessage)}</p>
      <div class="flex gap-2 mt-5 justify-end">
        <button id="modal-cancel" class="btn interactive rounded-lg border border-line px-4 py-2 text-[13px] font-medium text-text bg-bg hover:bg-surface">Cancel</button>
        <button id="modal-confirm" class="btn interactive rounded-lg px-4 py-2 text-[13px] font-medium bg-[#ff8d78]/10 text-[#ff8d78] border border-[#ff8d78]/20 hover:bg-[#ff8d78]/20">Delete</button>
      </div>
    `;
  } else if (modalType === "auth") {
    body = `
      <form id="modal-auth-form" class="mt-4 space-y-3">
        <div>
          <label class="block text-[10px] font-medium uppercase tracking-[0.12em] text-faint mb-1.5">API Key</label>
          <input name="api_key" type="password" required placeholder="sk-..." class="h-9 w-full rounded-lg border border-line bg-bg px-3 font-mono text-[12px] outline-none placeholder:text-faint focus:border-accent text-text" />
        </div>
        <div>
          <label class="block text-[10px] font-medium uppercase tracking-[0.12em] text-faint mb-1.5">Profile Name <span class="text-faint normal-case">(Optional)</span></label>
          <input name="name" placeholder="e.g. ${esc(modalProviderKind)}-prod" class="h-9 w-full rounded-lg border border-line bg-bg px-3 text-[13px] outline-none placeholder:text-faint focus:border-accent text-text" />
        </div>
        <div>
          <label class="block text-[10px] font-medium uppercase tracking-[0.12em] text-faint mb-1.5">Base URL <span class="text-faint normal-case">(Optional)</span></label>
          <input name="base_url" placeholder="https://..." class="h-9 w-full rounded-lg border border-line bg-bg px-3 font-mono text-[12px] outline-none placeholder:text-faint focus:border-accent text-text" />
        </div>
        <div class="flex gap-2 mt-5 justify-end">
          <button type="button" id="modal-cancel" class="btn interactive rounded-lg border border-line px-4 py-2 text-[13px] font-medium text-text bg-bg hover:bg-surface">Cancel</button>
          <button type="submit" class="btn interactive rounded-lg px-4 py-2 text-[13px] font-medium bg-accent text-bg btn-accent">Connect</button>
        </div>
      </form>
    `;
  } else if (modalType === "create-key") {
    const providerOpts = appState?.profiles.length
      ? appState.profiles.map((p) => `<option value="${esc(p.id)}">${esc(p.name)}</option>`).join("")
      : "<option value=\"\">No providers</option>";
    body = `
      <form id="modal-key-form" class="mt-4 space-y-3">
        <div>
          <label class="block text-[10px] font-medium uppercase tracking-[0.12em] text-faint mb-1.5">Key Name</label>
          <input name="name" required placeholder="e.g. production-key" class="h-9 w-full rounded-lg border border-line bg-bg px-3 text-[13px] outline-none placeholder:text-faint focus:border-accent text-text" />
        </div>
        <div>
          <label class="block text-[10px] font-medium uppercase tracking-[0.12em] text-faint mb-1.5">Provider</label>
          <select name="profile_id" class="h-9 w-full rounded-lg border border-line bg-bg px-3 text-[13px] outline-none focus:border-accent text-text appearance-none cursor-pointer">
            ${providerOpts}
          </select>
        </div>
        <div class="flex gap-2 mt-5 justify-end">
          <button type="button" id="modal-cancel" class="btn interactive rounded-lg border border-line px-4 py-2 text-[13px] font-medium text-text bg-bg hover:bg-surface">Cancel</button>
          <button type="submit" class="btn interactive rounded-lg px-4 py-2 text-[13px] font-medium bg-accent text-bg btn-accent">Create Key</button>
        </div>
      </form>
    `;
  }

  container.innerHTML = `
    <div class="modal-backdrop">
      <div class="modal-card">
        <div class="flex items-center justify-between mb-1">
          <h3 class="text-[15px] font-medium">${esc(modalTitle)}</h3>
          <button id="modal-cancel-x" class="text-faint hover:text-muted interactive p-1 rounded">${Icon.close}</button>
        </div>
        ${body}
      </div>
    </div>
  `;
}

function renderMain() {
  const content = $("main-content");
  const scroll = $("main-scroll");
  if (!content || !scroll) return;
  if (!appState) {
    content.innerHTML = renderLoading();
    return;
  }

  content.style.animation = "none";
  void content.offsetHeight;
  content.style.animation = "viewEnter 280ms var(--ease-out) both";

  let html = "";
  switch (currentView) {
    case "overview":
      html = renderOverview();
      break;
    case "providers":
      html = renderProviders();
      break;
    case "models":
      html = renderModels();
      break;
    case "keys":
      html = renderKeys();
      break;
    case "activity":
      html = renderActivity();
      break;
    case "playground":
      html = renderPlayground();
      break;
    default:
      html = renderOverview();
  }

  content.innerHTML = html;
  scroll.scrollTop = 0;
}

/* -------------------------------------------------------------------------- */
/*                                 Views                                      */
/* -------------------------------------------------------------------------- */

function renderLoading() {
  return `
    <div class="flex flex-col items-center justify-center h-[50vh] gap-5">
      <div class="h-14 w-14 text-faint" style="animation:logoPulse 2s ease-in-out infinite">${LogoIcon()}</div>
      <p class="text-sm text-muted animate-pulse">Loading dashboard...</p>
    </div>
  `;
}

/* ---- Overview ---- */
function renderOverview() {
  const s = appState!;
  const apiUrl = new URL(s.service.api_base_url, location.origin).toString();
  const isOnline = s.setup.traffic_ready;

  // Compute metrics
  const recentLogs = s.logs.slice(0, 20);
  const avgLatency = recentLogs.length
    ? Math.round(recentLogs.reduce((a, b) => a + b.duration_ms, 0) / recentLogs.length)
    : 0;
  const errorCount = recentLogs.filter((l) => (l.status_code ?? 0) >= 400 || !!l.error_message).length;
  const errorRate = recentLogs.length ? Math.round((errorCount / recentLogs.length) * 100) : 0;

  // Sparkline data (last 12 hours, bucketed)
  const now = new Date();
  const hours = Array.from({ length: 24 }, (_, i) => {
    const h = new Date(now.getTime() - (23 - i) * 60 * 60 * 1000);
    return { hour: h.getHours(), count: 0 };
  });
  s.logs.forEach((log) => {
    const d = new Date(log.started_at);
    const diff = (now.getTime() - d.getTime()) / (1000 * 60 * 60);
    if (diff >= 0 && diff < 24) {
      const idx = 23 - Math.floor(diff);
      if (idx >= 0 && idx < 24) hours[idx].count++;
    }
  });
  const maxCount = Math.max(1, ...hours.map((h) => h.count));

  const steps = [
    { label: "Connect provider", desc: "Add your first AI provider", done: s.setup.provider_ready },
    { label: "Sync models", desc: "Fetch available models", done: s.setup.models_ready },
    { label: "Create a key", desc: "Generate an API key", done: s.setup.key_ready },
    { label: "Send a request", desc: "Proxy your first call", done: s.setup.traffic_ready },
  ];
  const completedSteps = steps.filter((s) => s.done).length;

  return `
    <div>
      <!-- Status Hero -->
      <div class="rounded-2xl border border-line p-6 mb-6 ${isOnline ? "bg-[rgba(125,211,200,0.03)]" : "bg-surface"}" style="animation:fadeUp 300ms var(--ease-out) both">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div class="flex items-center gap-4">
            <div class="h-10 w-10 rounded-full flex items-center justify-center ${isOnline ? "bg-[#7dbeb3]/10" : "bg-surface border border-line"}">
              <span class="status-dot ${isOnline ? "bg-[#7dbeb3] pulse" : "bg-[#ff8d78]"}"></span>
            </div>
            <div>
              <div class="text-[18px] font-medium tracking-tight">${isOnline ? "System Online" : "Setup Required"}</div>
              <div class="text-[12px] text-muted mt-0.5 font-mono">${esc(apiUrl)}</div>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <button id="ov-copy-url" class="btn interactive flex items-center gap-1.5 rounded-lg border border-line px-3 py-2 text-[12px] font-medium bg-bg text-text hover:bg-surface">
              ${Icon.copy}
              Copy URL
            </button>
            ${!isOnline ? `<button data-nav="providers" class="btn-nav btn interactive rounded-lg px-3 py-2 text-[12px] font-medium bg-accent text-bg btn-accent">Setup Providers</button>` : ""}
          </div>
        </div>
      </div>

      <!-- Metrics -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 mb-6">
        ${[
          { value: s.counts.logs, label: "Total Requests", color: "#7dbeb3", icon: Icon.activity },
          { value: s.counts.profiles, label: "Providers", color: "var(--color-accent)", icon: Icon.providers },
          { value: s.counts.models, label: "Models", color: "#8a9ec9", icon: Icon.models },
          { value: s.counts.keys, label: "API Keys", color: "#c98a9e", icon: Icon.keys },
        ]
          .map(
            (m) => `
          <div class="rounded-xl border border-line p-4 bg-surface" style="animation:fadeUp 280ms var(--ease-out) both">
            <div class="flex items-center gap-2 mb-3">
              <span class="opacity-60">${m.icon}</span>
              <span class="text-[10px] font-medium uppercase tracking-[0.12em] text-faint">${m.label}</span>
            </div>
            <div class="font-mono text-[26px] font-medium leading-none text-text">${m.value}</div>
          </div>
        `,
          )
          .join("")}
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
        <!-- Activity Sparkline -->
        <div class="lg:col-span-2 rounded-xl border border-line p-5 bg-surface" style="animation:fadeUp 300ms var(--ease-out) both; animation-delay:50ms">
          <div class="flex items-center justify-between mb-4">
            <div>
              <div class="text-[15px] font-medium tracking-tight">Activity</div>
              <div class="text-[12px] text-muted mt-0.5">Last 24 hours</div>
            </div>
            <div class="flex items-center gap-4 text-[11px] text-muted">
              <span class="flex items-center gap-1"><span class="status-dot bg-[#7dbeb3]"></span> Avg ${avgLatency}ms</span>
              <span class="flex items-center gap-1"><span class="status-dot bg-[#ff8d78]"></span> ${errorRate}% errors</span>
            </div>
          </div>
          <div class="flex items-end gap-[3px] h-16">
            ${hours
              .map(
                (h) => `
              <div class="spark-bar flex-1" style="height:${Math.max(4, (h.count / maxCount) * 100)}%" title="${h.hour}:00 — ${h.count} requests"></div>
            `,
              )
              .join("")}
          </div>
          <div class="flex justify-between mt-2 text-[10px] text-faint font-mono">
            <span>24h ago</span>
            <span>Now</span>
          </div>
        </div>

        <!-- Provider Quick Status -->
        <div class="rounded-xl border border-line p-5 bg-surface" style="animation:fadeUp 300ms var(--ease-out) both; animation-delay:100ms">
          <div class="text-[15px] font-medium tracking-tight mb-4">Providers</div>
          <div class="space-y-2">
            ${s.providers
              .map((pt) => {
                const profile = s.profiles.find((p) => p.provider === pt.kind);
                const connected = !!profile;
                return `
              <div class="flex items-center justify-between py-1.5">
                <div class="flex items-center gap-2.5">
                  <span class="status-dot ${connected ? "bg-[#7dbeb3]" : "bg-faint"}"></span>
                  <span class="text-[13px] ${connected ? "text-text" : "text-muted"}">${esc(pt.kind)}</span>
                </div>
                ${connected ? `<span class="text-[11px] font-mono text-faint">${profile.model_count} models</span>` : `<span class="text-[11px] text-faint">Disconnected</span>`}
              </div>
            `;
              })
              .join("")}
          </div>
          <button data-nav="providers" class="btn-nav mt-4 w-full text-center text-[12px] text-muted hover:text-text interactive py-1.5 rounded-lg border border-line hover:bg-surface">
            Manage Providers
          </button>
        </div>
      </div>

      <!-- Recent Activity + Setup -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Recent Requests -->
        <div class="rounded-xl border border-line p-5 bg-surface" style="animation:fadeUp 300ms var(--ease-out) both; animation-delay:150ms">
          <div class="flex items-center justify-between mb-4">
            <div class="text-[15px] font-medium tracking-tight">Recent Requests</div>
            <button data-nav="activity" class="btn-nav text-[11px] text-muted hover:text-text interactive flex items-center gap-1">
              View All ${Icon.chevronRight}
            </button>
          </div>
          <div class="space-y-0">
            ${s.logs.slice(0, 6).length === 0
              ? `<div class="text-[13px] text-muted py-4 text-center">No requests yet</div>`
              : s.logs
                  .slice(0, 6)
                  .map(
                    (log) => {
                      const err = (log.status_code ?? 0) >= 400 || !!log.error_message;
                      return `
                <div class="flex items-center justify-between py-2.5 border-b border-line last:border-0">
                  <div class="flex items-center gap-2.5 min-w-0">
                    <span class="status-dot shrink-0 ${err ? "bg-[#ff8d78]" : "bg-[#7dbeb3]"}"></span>
                    <div class="min-w-0">
                      <div class="text-[12px] font-mono text-text truncate">${esc(log.model)}</div>
                      <div class="text-[10px] text-faint">${esc(log.profile_name || log.provider)}</div>
                    </div>
                  </div>
                  <div class="text-right shrink-0 ml-3">
                    <div class="text-[11px] text-muted">${fmtRelative(log.started_at)}</div>
                    <div class="text-[10px] font-mono text-faint">${log.duration_ms}ms</div>
                  </div>
                </div>
              `;
                    },
                  )
                  .join("")}
          </div>
        </div>

        <!-- Setup Progress -->
        ${completedSteps < 4
          ? `
        <div class="rounded-xl border border-line p-5 bg-surface" style="animation:fadeUp 300ms var(--ease-out) both; animation-delay:200ms">
          <div class="flex items-center justify-between mb-4">
            <div class="text-[15px] font-medium tracking-tight">Setup</div>
            <div class="text-[11px] text-muted">${completedSteps} of 4</div>
          </div>
          <div class="space-y-0">
            ${steps
              .map(
                (step) => `
              <div class="flex items-center gap-3 py-2.5 border-b border-line last:border-0">
                <div class="h-6 w-6 rounded-full flex items-center justify-center shrink-0 ${step.done ? "bg-[#7dbeb3]/10 text-[#7dbeb3]" : "border border-line text-faint"}">
                  ${step.done ? Icon.check : `<span class="text-[10px] font-mono">${steps.indexOf(step) + 1}</span>`}
                </div>
                <div class="min-w-0">
                  <div class="text-[13px] font-medium ${step.done ? "text-text" : "text-muted"}">${step.label}</div>
                  <div class="text-[11px] text-faint">${step.desc}</div>
                </div>
              </div>
            `,
              )
              .join("")}
          </div>
        </div>
      `
          : ""}
      </div>
    </div>
  `;
}

/* ---- Providers ---- */
function renderProviders() {
  const s = appState!;

  return `
    <div>
      <div class="mb-6">
        <h1 class="text-xl font-medium tracking-tight">Providers</h1>
        <p class="text-sm text-muted mt-0.5">Enable and manage your AI provider connections</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        ${s.providers
          .map((pt, i) => {
            const profile = s.profiles.find((p) => p.provider === pt.kind);
            const connected = !!profile;
            return `
          <div class="provider-card ${connected ? "" : "opacity-75"}" style="animation:fadeUp 200ms var(--ease-out) both; animation-delay:${i * 40}ms">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <div class="h-10 w-10 rounded-xl flex items-center justify-center border border-line bg-bg shrink-0">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">${Icon.providers.replace(/width="15" height="15"/, "")}</svg>
                </div>
                <div>
                  <div class="text-[15px] font-medium text-text">${esc(pt.kind)}</div>
                  <div class="flex items-center gap-1.5 mt-0.5">
                    <span class="status-dot ${connected ? "bg-[#7dbeb3]" : "bg-faint"}"></span>
                    <span class="text-[11px] ${connected ? "text-[#7dbeb3]" : "text-faint"}">${connected ? "Connected" : "Disconnected"}</span>
                  </div>
                </div>
              </div>
              <div class="provider-toggle toggle-track ${connected ? "on" : ""}" data-provider="${esc(pt.kind)}" data-profile-id="${profile?.id || ""}" role="switch" aria-checked="${connected}"></div>
            </div>

            ${connected && profile
              ? `
            <div class="border-t border-line pt-3 mt-3">
              <div class="flex items-center justify-between text-[12px]">
                <div class="text-muted">
                  <span class="font-mono text-text">${profile.model_count}</span> models synced
                </div>
                <div class="font-mono text-faint text-[11px]">${esc(profile.selector)}</div>
              </div>
              <div class="flex items-center gap-2 mt-3">
                <button class="p-act btn interactive rounded-lg border border-line px-3 py-1.5 text-[11px] font-medium bg-bg text-text hover:bg-surface" data-id="${profile.id}" data-a="sync">Sync Models</button>
                <button class="p-act btn interactive rounded-lg border border-line px-3 py-1.5 text-[11px] font-medium bg-bg text-text hover:bg-surface" data-id="${profile.id}" data-a="auth">Re-Auth</button>
                <button class="p-del btn interactive rounded-lg border border-line px-3 py-1.5 text-[11px] font-medium bg-bg text-[#ff8d78] hover:bg-[rgba(255,141,120,0.08)] ml-auto" data-id="${profile.id}" data-name="${esc(profile.name)}">Disconnect</button>
              </div>
            </div>
            `
              : `
            <div class="border-t border-line pt-3 mt-3">
              <p class="text-[12px] text-muted mb-3">Connect this provider to start using its models through Gunmetal.</p>
              <button class="provider-connect btn interactive rounded-lg px-3 py-1.5 text-[12px] font-medium bg-accent text-bg btn-accent" data-provider="${esc(pt.kind)}">Connect Provider</button>
            </div>
            `}
          </div>
        `;
          })
          .join("")}
      </div>
    </div>
  `;
}

/* ---- Models ---- */
function renderModels() {
  const s = appState!;
  const q = modelSearch.trim().toLowerCase();
  const filtered = q ? s.models.filter((m) => m.id.toLowerCase().includes(q) || m.provider.toLowerCase().includes(q)) : s.models;

  return `
    <div>
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-6">
        <div>
          <h1 class="text-xl font-medium tracking-tight">Models</h1>
          <p class="text-sm text-muted mt-0.5">${s.models.length} models available across ${s.profiles.length} providers</p>
        </div>
        <div class="relative">
          <span class="absolute left-3 top-1/2 -translate-y-1/2 text-faint">${Icon.search}</span>
          <input id="mod-q" value="${esc(modelSearch)}" placeholder="Search models..." class="h-9 w-full sm:w-56 rounded-lg border border-line bg-bg pl-9 pr-3 text-[13px] outline-none placeholder:text-faint focus:border-accent text-text" />
        </div>
      </div>

      ${filtered.length === 0
        ? `
        <div class="rounded-xl border border-dashed border-line p-12 text-center">
          <p class="text-sm text-muted font-medium">No models found</p>
          <p class="text-[13px] text-faint mt-1">${q ? "Try a different search term." : "Sync models from your providers first."}</p>
        </div>
      `
        : `
        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
          ${filtered
            .slice(0, 60)
            .map(
              (m, i) => `
            <div class="flex items-center justify-between rounded-lg border border-line px-4 py-3 transition-colors hover:bg-surface group" style="animation:fadeUp 160ms var(--ease-out) both; animation-delay:${i * 15}ms">
              <div class="min-w-0">
                <div class="truncate font-mono text-[12px] font-medium text-text">${esc(m.id)}</div>
                <div class="flex items-center gap-2 mt-1">
                  <span class="text-[10px] uppercase tracking-wide text-faint">${esc(m.provider)}</span>
                  ${m.supports_reasoning ? '<span class="rounded bg-surface border border-line px-1.5 py-[1px] text-[9px] font-medium uppercase tracking-[0.05em] text-muted">reason</span>' : ""}
                  ${m.supports_tools ? '<span class="rounded bg-surface border border-line px-1.5 py-[1px] text-[9px] font-medium uppercase tracking-[0.05em] text-muted">tools</span>' : ""}
                </div>
              </div>
              <button class="opacity-0 group-hover:opacity-100 interactive text-faint hover:text-text p-1.5 rounded transition-opacity" title="Copy model ID" onclick="navigator.clipboard.writeText('${esc(m.id)}').then(()=>{showToast('Copied model ID')}).catch(()=>{})">
                ${Icon.copy}
              </button>
            </div>
          `,
            )
            .join("")}
        </div>
        ${filtered.length > 60 ? `<p class="mt-4 text-[11px] text-center text-faint font-mono">Showing 60 of ${filtered.length} models</p>` : ""}
      `}
    </div>
  `;
}

/* ---- API Keys ---- */
function renderKeys() {
  const s = appState!;

  return `
    <div>
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-xl font-medium tracking-tight">API Keys</h1>
          <p class="text-sm text-muted mt-0.5">Manage access keys for your applications</p>
        </div>
        <button id="btn-create-key" class="btn interactive flex items-center gap-1.5 rounded-lg px-4 py-2 text-[13px] font-medium bg-accent text-bg btn-accent">
          ${Icon.plus}
          Create API Key
        </button>
      </div>

      ${s.keys.length === 0
        ? `
        <div class="rounded-xl border border-dashed border-line p-12 text-center">
          <div class="h-10 w-10 text-faint mx-auto mb-3">${Icon.keys}</div>
          <p class="text-sm text-muted font-medium">No API keys</p>
          <p class="text-[13px] text-faint mt-1">Create a key to start using Gunmetal.</p>
        </div>
      `
        : `
        <div class="rounded-xl border border-line overflow-hidden">
          <div class="key-row bg-surface text-[10px] font-medium uppercase tracking-[0.12em] text-faint">
            <div>Name</div>
            <div>Key</div>
            <div>Providers</div>
            <div></div>
          </div>
          ${s.keys
            .map(
              (k) => `
            <div class="key-row">
              <div class="text-[13px] font-medium text-text">${esc(k.name)}</div>
              <div class="flex items-center gap-2">
                <code class="font-mono text-[12px] text-muted">${esc(k.prefix)}...</code>
                <button class="key-copy btn interactive text-faint hover:text-text p-1 rounded" title="Copy key" data-prefix="${esc(k.prefix)}">
                  ${Icon.copy}
                </button>
              </div>
              <div class="text-[12px] text-muted truncate">${k.providers.length ? esc(k.providers.join(", ")) : '<span class="text-faint italic">All</span>'}</div>
              <div class="flex items-center justify-end">
                <button class="key-del btn interactive text-faint hover:text-[#ff8d78] p-1.5 rounded hover:bg-[rgba(255,141,120,0.08)]" title="Delete" data-id="${k.id}" data-name="${esc(k.name)}">
                  ${Icon.trash}
                </button>
              </div>
            </div>
          `,
            )
            .join("")}
        </div>
      `}
    </div>
  `;
}

/* ---- Activity ---- */
function renderActivity() {
  const s = appState!;
  const f = activityFilters;
  const filtered = s.logs.filter((log) => {
    const providerMatch = f.provider === "all" || `${log.provider}::${log.profile_name || ""}` === f.provider;
    const isError = (log.status_code ?? 0) >= 400 || !!log.error_message;
    const statusMatch = f.status === "all" || (f.status === "success" ? !isError : isError);
    const q = f.query.trim().toLowerCase();
    const searchMatch =
      !q ||
      [log.provider, log.profile_name || "", log.model, log.endpoint, log.key_name || "", log.request_mode || "", log.error_message || ""]
        .join(" ")
        .toLowerCase()
        .includes(q);
    return providerMatch && statusMatch && searchMatch;
  });

  const limit = f.limit === "all" ? filtered.length : Number(f.limit) || 8;
  const display = filtered.slice(0, limit);

  const providerOpts =
    '<option value="all">All providers</option>' +
    s.provider_summaries
      .map(
        (ps) =>
          `<option value="${esc(`${ps.provider}::${ps.profile_name || ""}`)}"${f.provider === `${ps.provider}::${ps.profile_name || ""}` ? " selected" : ""}>${esc(ps.label)}</option>`,
      )
      .join("");

  return `
    <div>
      <div class="mb-6">
        <h1 class="text-xl font-medium tracking-tight">Activity</h1>
        <p class="text-sm text-muted mt-0.5">Request logs and metrics</p>
      </div>

      <div class="flex flex-wrap items-center gap-2 mb-5 p-2 rounded-xl border border-line bg-surface">
        <div class="relative flex-1 min-w-[180px]">
          <span class="absolute left-3 top-1/2 -translate-y-1/2 text-faint">${Icon.search}</span>
          <input id="filt-q" value="${esc(f.query)}" placeholder="Search requests..." class="h-8 w-full rounded-lg bg-transparent pl-9 pr-3 text-[13px] outline-none placeholder:text-faint focus:bg-surface text-text" />
        </div>
        <div class="h-4 w-px bg-line hidden sm:block"></div>
        <select id="filt-p" class="h-8 rounded-lg bg-transparent px-2.5 text-[13px] outline-none cursor-pointer hover:bg-bg text-muted border border-transparent hover:border-line">
          ${providerOpts}
        </select>
        <select id="filt-s" class="h-8 rounded-lg bg-transparent px-2.5 text-[13px] outline-none cursor-pointer hover:bg-bg text-muted border border-transparent hover:border-line">
          <option value="all"${f.status === "all" ? " selected" : ""}>All</option>
          <option value="success"${f.status === "success" ? " selected" : ""}>Success</option>
          <option value="error"${f.status === "error" ? " selected" : ""}>Error</option>
        </select>
        <select id="filt-l" class="h-8 rounded-lg bg-transparent px-2.5 text-[13px] outline-none cursor-pointer hover:bg-bg text-muted border border-transparent hover:border-line">
          <option value="8"${f.limit === "8" ? " selected" : ""}>Last 8</option>
          <option value="24"${f.limit === "24" ? " selected" : ""}>Last 24</option>
          <option value="all"${f.limit === "all" ? " selected" : ""}>All</option>
        </select>
      </div>

      ${display.length === 0
        ? `
        <div class="rounded-xl border border-dashed border-line p-12 text-center">
          <p class="text-sm text-muted font-medium">No activity to show</p>
        </div>
      `
        : `
        <div class="rounded-xl border border-line overflow-hidden">
          <div class="activity-row bg-surface text-[10px] font-medium uppercase tracking-[0.12em] text-faint border-b border-line">
            <div></div>
            <div>Model & Route</div>
            <div>Provider</div>
            <div>Time</div>
            <div class="text-right">Metrics</div>
          </div>
          ${display
            .map((log) => {
              const isError = (log.status_code ?? 0) >= 400 || !!log.error_message;
              return `
            <div class="activity-row">
              <div><span class="status-dot ${isError ? "bg-[#ff8d78]" : "bg-[#7dbeb3]"}"></span></div>
              <div class="min-w-0">
                <div class="font-mono text-[11px] font-medium text-text truncate">${esc(log.model)}</div>
                <div class="text-[10px] text-faint">${esc(log.request_mode || log.endpoint)}</div>
              </div>
              <div class="text-[12px] text-muted truncate">${esc(log.profile_name || log.provider)}</div>
              <div class="text-[12px] text-muted" title="${fmtDate(log.started_at)}">${fmtRelative(log.started_at)}</div>
              <div class="text-right">
                <div class="font-mono text-[11px] text-text">${log.duration_ms}ms</div>
                ${log.total_tokens ? `<div class="text-[10px] font-mono text-faint">${log.total_tokens} tok</div>` : ""}
              </div>
            </div>
            ${isError
              ? `
            <div class="px-4 pb-3 bg-[rgba(255,141,120,0.02)] border-b border-line">
              <div class="font-mono text-[11px] text-[#ff8d78]/80 whitespace-pre-wrap pl-6 border-l-2 border-[#ff8d78]/20 py-1">${esc(log.error_message || `HTTP ${log.status_code}`)}</div>
            </div>
            `
              : ""}
          `;
            })
            .join("")}
        </div>
        <p class="mt-3 text-[11px] text-center text-faint font-mono">Showing ${display.length} of ${filtered.length} logs</p>
      `}
    </div>
  `;
}

/* ---- Playground ---- */
function renderPlayground() {
  const s = appState!;
  const providers = s.profiles.filter((p) => s.models.some((m) => m.provider === p.provider));
  const models = s.models.filter((m) => m.provider === pgState.provider);

  return `
    <div class="flex flex-col" style="min-height:600px; height:calc(100dvh - 40px)">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4 shrink-0">
        <div>
          <h1 class="text-xl font-medium tracking-tight">Playground</h1>
          <p class="text-sm text-muted mt-0.5">Test your providers interactively</p>
        </div>
        ${pgState.messages.length > 0 && !pgState.running ? `<button id="pg-clr" class="btn interactive text-[12px] text-muted hover:text-text px-3 py-1.5 rounded-lg border border-line bg-bg hover:bg-surface">Clear Chat</button>` : ""}
      </div>

      <!-- Config Bar -->
      <div class="flex flex-wrap items-center gap-2 mb-4 shrink-0 p-2 rounded-xl border border-line bg-surface">
        <div class="relative flex-1 min-w-[140px]">
          <input id="pg-key" value="${esc(pgState.key)}" type="password" placeholder="API Key (gm_...)" class="h-8 w-full rounded-lg bg-transparent px-3 font-mono text-[11px] outline-none placeholder:text-faint focus:bg-bg text-text border border-transparent focus:border-line" />
        </div>
        <div class="h-4 w-px bg-line hidden sm:block"></div>
        <select id="pg-p" class="h-8 rounded-lg bg-transparent px-2 text-[12px] outline-none cursor-pointer hover:bg-bg text-text border border-transparent hover:border-line">
          ${providers.length ? providers.map((p) => `<option value="${esc(p.provider)}"${pgState.provider === p.provider ? " selected" : ""}>${esc(p.name)}</option>`).join("") : "<option>No providers</option>"}
        </select>
        <select id="pg-m" class="h-8 rounded-lg bg-transparent px-2 font-mono text-[11px] outline-none cursor-pointer hover:bg-bg text-text border border-transparent hover:border-line max-w-[200px]">
          ${models.length ? models.map((m) => `<option value="${esc(m.id)}"${pgState.model === m.id ? " selected" : ""}>${esc(m.id)}</option>`).join("") : "<option>No models</option>"}
        </select>
        <select id="pg-mode" class="h-8 rounded-lg bg-transparent px-2 text-[12px] outline-none cursor-pointer hover:bg-bg text-muted border border-transparent hover:border-line">
          <option value="chat"${pgState.mode === "chat" ? " selected" : ""}>Chat</option>
          <option value="responses"${pgState.mode === "responses" ? " selected" : ""}>Responses</option>
        </select>
      </div>

      <!-- Messages -->
      <div class="flex-1 overflow-y-auto rounded-xl border border-line bg-sidebar p-5 space-y-5 mb-4 min-h-[300px]">
        ${pgState.messages.length
          ? pgState.messages
              .map(
                (msg) => `
              <div class="flex ${msg.role === "user" ? "justify-end" : "justify-start"}" style="animation:fadeUp 200ms var(--ease-out) both">
                <div class="max-w-[85%]">
                  <div class="flex items-center gap-2 mb-1.5 ${msg.role === "user" ? "justify-end" : ""}">
                    <span class="h-5 w-5 rounded flex items-center justify-center text-[10px] font-bold text-bg ${msg.role === "user" ? "bg-accent" : "bg-[#7dbeb3]"}">${msg.role.charAt(0).toUpperCase()}</span>
                    <span class="text-[10px] font-medium uppercase tracking-[0.1em] text-faint">${msg.role}</span>
                  </div>
                  <div class="chat-${msg.role} rounded-2xl px-4 py-3 text-[14px] leading-relaxed whitespace-pre-wrap">${esc(msg.content)}</div>
                </div>
              </div>
            `,
              )
              .join("")
          : '<div class="h-full flex items-center justify-center text-[13px] text-muted">Select a model and send a message to test.</div>'}
        ${pgState.running ? `
          <div class="flex justify-start">
            <div class="max-w-[85%]">
              <div class="flex items-center gap-2 mb-1.5">
                <span class="h-5 w-5 rounded flex items-center justify-center text-[10px] font-bold text-bg bg-[#7dbeb3]">A</span>
                <span class="text-[10px] font-medium uppercase tracking-[0.1em] text-faint">assistant</span>
              </div>
              <div class="chat-assistant rounded-2xl px-4 py-3">
                <div class="flex gap-1">
                  <span class="w-2 h-2 rounded-full bg-faint animate-bounce" style="animation-delay:0ms"></span>
                  <span class="w-2 h-2 rounded-full bg-faint animate-bounce" style="animation-delay:150ms"></span>
                  <span class="w-2 h-2 rounded-full bg-faint animate-bounce" style="animation-delay:300ms"></span>
                </div>
              </div>
            </div>
          </div>
        ` : ""}
      </div>

      <!-- Input -->
      <form id="pg-form" class="shrink-0 flex gap-2 relative">
        <textarea id="pg-in" placeholder="Message..." rows="1" class="min-h-[48px] flex-1 resize-y rounded-xl border border-line bg-sidebar px-4 py-3 text-[14px] outline-none placeholder:text-faint focus:border-accent text-text shadow-sm" ${pgState.running ? "disabled" : ""}></textarea>
        <button type="submit" class="btn interactive h-[48px] w-[48px] rounded-xl flex items-center justify-center bg-accent text-bg shadow-sm btn-accent shrink-0" ${pgState.running ? "disabled" : ""}>
          ${pgState.running
            ? '<svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/></svg>'
            : Icon.send}
        </button>
      </form>
    </div>
  `;
}

/* -------------------------------------------------------------------------- */
/*                                Events                                      */
/* -------------------------------------------------------------------------- */

function bindEvents() {
  // Sidebar nav
  document.querySelectorAll("[data-nav]").forEach((el) => {
    el.addEventListener("click", () => {
      const view = el.getAttribute("data-nav");
      if (view) navigate(view);
    });
  });

  // Nav buttons inside views
  document.querySelectorAll(".btn-nav").forEach((el) => {
    el.addEventListener("click", () => {
      const view = el.getAttribute("data-nav");
      if (view) navigate(view);
    });
  });

  // Refresh
  $("btn-refresh")?.addEventListener("click", () => loadState());

  // Copy URL
  $("btn-copy-url")?.addEventListener("click", async () => {
    if (!appState) return;
    const url = new URL(appState.service.api_base_url, location.origin).toString();
    const ok = await copyToClipboard(url);
    showToast(ok ? "Copied API URL" : "Copy failed", ok);
  });

  // Overview copy URL
  $("ov-copy-url")?.addEventListener("click", async () => {
    if (!appState) return;
    const url = new URL(appState.service.api_base_url, location.origin).toString();
    const ok = await copyToClipboard(url);
    showToast(ok ? "Copied API URL" : "Copy failed", ok);
  });

  // Modal events
  $("modal-cancel")?.addEventListener("click", closeModal);
  $("modal-cancel-x")?.addEventListener("click", closeModal);
  $("modal-confirm")?.addEventListener("click", () => {
    if (modalOnConfirm) modalOnConfirm();
    closeModal();
  });
  $("modal-auth-form")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const data = new FormData(form);
    try {
      const res = await api.createProfile({
        provider: modalProviderKind,
        name: String(data.get("name") || modalProviderKind),
        api_key: String(data.get("api_key")),
        base_url: String(data.get("base_url")),
      });
      showToast((res.message as string) || "Provider connected");
      closeModal();
      loadState();
    } catch (err) {
      showToast((err as Error).message, false);
    }
  });
  $("modal-key-form")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const data = new FormData(form);
    const profileId = String(data.get("profile_id"));
    const name = String(data.get("name"));
    if (!profileId || !name) return;
    try {
      const res = await api.createKey(profileId, name);
      showToast(res.message);
      if (res.secret) {
        pgState.key = res.secret;
        showToast("Key created — copied to playground", true);
      }
      closeModal();
      loadState();
    } catch (err) {
      showToast((err as Error).message, false);
    }
  });

  // Providers view
  if (currentView === "providers") {
    // Toggle switches
    document.querySelectorAll<HTMLElement>(".provider-toggle").forEach((el) => {
      el.addEventListener("click", () => {
        const provider = el.dataset.provider!;
        const profileId = el.dataset.profileId;
        if (profileId) {
          // Disable / disconnect
          const profile = appState?.profiles.find((p) => p.id === profileId);
          openConfirm("Disconnect Provider", `Disconnect ${profile?.name || provider}?`, () => {
            api
              .deleteProfile(profileId)
              .then((res) => {
                showToast(res.message);
                loadState();
              })
              .catch((e) => showToast((e as Error).message, false));
          });
        } else {
          // Enable / connect
          openAuthModal(provider);
        }
      });
    });

    // Connect buttons for disconnected providers
    document.querySelectorAll<HTMLElement>(".provider-connect").forEach((el) => {
      el.addEventListener("click", () => {
        openAuthModal(el.dataset.provider!);
      });
    });

    // Profile actions
    document.querySelectorAll<HTMLElement>(".p-act").forEach((el) => {
      el.addEventListener("click", async () => {
        const id = el.dataset.id!;
        const action = el.dataset.a!;
        try {
          const res = await api.action(id, action);
          if ((res as { auth_url?: string }).auth_url) {
            window.open((res as { auth_url: string }).auth_url, "_blank", "noopener,noreferrer");
          }
          showToast((res.message as string) || "Done");
          loadState();
        } catch (e) {
          showToast((e as Error).message, false);
        }
      });
    });

    document.querySelectorAll<HTMLElement>(".p-del").forEach((el) => {
      el.addEventListener("click", () => {
        const id = el.dataset.id!;
        const name = el.dataset.name!;
        openConfirm("Disconnect Provider", `Disconnect "${name}"? This will remove all synced models.`, () => {
          api
            .deleteProfile(id)
            .then((res) => {
              showToast(res.message);
              loadState();
            })
            .catch((e) => showToast((e as Error).message, false));
        });
      });
    });
  }

  // Models search
  if (currentView === "models") {
    $("mod-q")?.addEventListener("input", (e) => {
      modelSearch = (e.target as HTMLInputElement).value;
      refresh();
    });
  }

  // Keys view
  if (currentView === "keys") {
    $("btn-create-key")?.addEventListener("click", () => {
      openCreateKeyModal();
    });

    document.querySelectorAll<HTMLElement>(".key-copy").forEach((el) => {
      el.addEventListener("click", async () => {
        const prefix = el.dataset.prefix;
        showToast("Key prefix copied");
      });
    });

    document.querySelectorAll<HTMLElement>(".key-del").forEach((el) => {
      el.addEventListener("click", () => {
        const id = el.dataset.id!;
        const name = el.dataset.name!;
        openDeleteKeyModal(id, name);
      });
    });
  }

  // Activity filters
  if (currentView === "activity") {
    $("filt-p")?.addEventListener("change", (e) => {
      activityFilters.provider = (e.target as HTMLSelectElement).value;
      refresh();
    });
    $("filt-s")?.addEventListener("change", (e) => {
      activityFilters.status = (e.target as HTMLSelectElement).value;
      refresh();
    });
    $("filt-l")?.addEventListener("change", (e) => {
      activityFilters.limit = (e.target as HTMLSelectElement).value;
      refresh();
    });
    $("filt-q")?.addEventListener("input", (e) => {
      activityFilters.query = (e.target as HTMLInputElement).value;
      refresh();
    });
  }

  // Playground
  if (currentView === "playground") {
    $("pg-p")?.addEventListener("change", (e) => {
      pgState.provider = (e.target as HTMLSelectElement).value;
      const models = appState!.models.filter((m) => m.provider === pgState.provider);
      pgState.model = models[0]?.id || "";
      refresh();
    });

    $("pg-key")?.addEventListener("change", (e) => {
      pgState.key = (e.target as HTMLInputElement).value;
    });

    $("pg-m")?.addEventListener("change", (e) => {
      pgState.model = (e.target as HTMLSelectElement).value;
    });

    $("pg-mode")?.addEventListener("change", (e) => {
      pgState.mode = (e.target as HTMLSelectElement).value as "chat" | "responses";
    });

    $("pg-clr")?.addEventListener("click", () => {
      pgState.messages = [];
      pgState.usage = null;
      refresh();
    });

    $("pg-form")?.addEventListener("submit", async (e) => {
      e.preventDefault();
      const input = $("pg-in") as HTMLTextAreaElement;
      const text = input.value.trim();
      if (!text || pgState.running || !pgState.model) return;

      pgState.messages.push({ role: "user", content: text });
      pgState.messages.push({ role: "assistant", content: "" });
      pgState.running = true;
      input.value = "";
      input.style.height = "auto";
      refresh();

      const messages = pgState.messages
        .filter((m) => m.role !== "assistant" || m.content)
        .map((m) => ({ role: m.role, content: m.content }));

      try {
        const res = await api.chat(pgState.mode, pgState.key, pgState.model, messages);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const reader = res.body?.getReader();
        if (!reader) throw new Error("No response body");

        const decoder = new TextDecoder();
        let buffer = "";
        let content = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          let idx;
          while ((idx = buffer.indexOf("\n\n")) !== -1) {
            const chunk = buffer.slice(0, idx);
            buffer = buffer.slice(idx + 2);
            const lines = chunk.split(/\r?\n/);
            let event = "message";
            const dataLines: string[] = [];
            for (const line of lines) {
              if (line.startsWith("event:")) event = line.slice(6).trim();
              else if (line.startsWith("data:")) dataLines.push(line.slice(5).trimStart());
            }
            if (dataLines.length && dataLines.join("") !== "[DONE]") {
              const data = JSON.parse(dataLines.join("\n"));
              if (pgState.mode === "chat") {
                const delta = data?.choices?.[0]?.delta?.content || "";
                if (delta) content += delta;
              } else {
                if (event === "response.output_text.delta") content += data.delta || "";
                else if (event === "response.completed") {
                  content = data?.response?.output_text || content;
                  pgState.usage = data?.response?.usage || null;
                }
              }
              const last = pgState.messages[pgState.messages.length - 1];
              if (last && last.role === "assistant") last.content = content;
              refresh();
            }
          }
        }
        showToast("Done");
      } catch (err) {
        pgState.messages = pgState.messages.slice(0, Math.max(0, pgState.messages.length - 2));
        showToast((err as Error).message, false);
      } finally {
        pgState.running = false;
        refresh();
      }
    });

    const textarea = $("pg-in") as HTMLTextAreaElement | null;
    if (textarea) {
      textarea.addEventListener("input", () => {
        textarea.style.height = "auto";
        textarea.style.height = textarea.scrollHeight + "px";
      });
      textarea.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          $("pg-form")?.dispatchEvent(new Event("submit"));
        }
      });
    }
  }
}

/* -------------------------------------------------------------------------- */
/*                                Router                                      */
/* -------------------------------------------------------------------------- */

function navigate(view: string) {
  currentView = view;
  refresh();
}

function refresh() {
  renderSidebar();
  renderMain();
  renderModal();
  bindEvents();
}

/* -------------------------------------------------------------------------- */
/*                                Load State                                  */
/* -------------------------------------------------------------------------- */

async function loadState() {
  try {
    appState = await api.state();
    if (!pgState.provider && appState.profiles.length) {
      pgState.provider = appState.profiles[0].provider;
    }
    if (!pgState.model && appState.models.length) {
      const models = appState.models.filter((m) => m.provider === pgState.provider);
      pgState.model = models[0]?.id || appState.models[0].id;
    }
  } catch {
    appState = null;
  }
  if (!$("sidebar-nav")) renderApp();
  refresh();
}

/* -------------------------------------------------------------------------- */
/*                                 Init                                       */
/* -------------------------------------------------------------------------- */

renderApp();
loadState();

// Auto-refresh every 30s
setInterval(() => {
  if (!pgState.running) loadState();
}, 30000);
