import "./styles.css";

/* ─────────────── API ─────────────── */
const API = "/webui/api";
async function fetchJson(url: string, opts?: RequestInit) {
  const r = await fetch(url, { headers: { "Content-Type": "application/json" }, ...opts });
  const t = await r.text();
  const b = t ? JSON.parse(t) : {};
  if (!r.ok) throw new Error(b?.error?.message || b?.message || "Failed");
  return b;
}
const api = {
  state: () => fetchJson(`${API}/state`),
  createProfile: (body: object) => fetchJson(`${API}/profiles`, { method: "POST", body: JSON.stringify(body) }),
  action: (id: string, act: string) => fetchJson(`${API}/profiles/${id}/${act}`, { method: "POST" }),
  createKey: (pid: string, name: string) => fetchJson(`${API}/profiles/${pid}/keys`, { method: "POST", body: JSON.stringify({ name }) }),
  setKeyState: (id: string, state: string) => fetchJson(`${API}/keys/${id}/state`, { method: "POST", body: JSON.stringify({ state }) }),
  deleteKey: (id: string) => fetchJson(`${API}/keys/${id}`, { method: "DELETE" }),
  deleteProfile: (id: string) => fetchJson(`${API}/profiles/${id}`, { method: "DELETE" }),
  chat: (mode: string, key: string, model: string, messages: any[]) =>
    fetch(mode === "responses" ? "/v1/responses" : "/v1/chat/completions", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: `Bearer ${key}` },
      body: JSON.stringify(
        mode === "responses"
          ? { model, stream: true, input: messages.map((m) => ({ role: m.role === "system" ? "developer" : m.role, content: [{ type: "input_text", text: m.content }] })) }
          : { model, stream: true, messages }
      ),
    }),
};

/* ─────────────── Utils ─────────────── */
const $ = (id: string) => document.getElementById(id) as HTMLElement | null;
const esc = (v: unknown) => String(v ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const fmtTime = (v: string) => { const d = new Date(v); return isNaN(d.getTime()) ? v : d.toLocaleString(); };
const fmtRel = (v: string) => {
  const d = new Date(v); if (isNaN(d.getTime())) return v;
  const s = Math.floor((Date.now() - d.getTime()) / 1000);
  if (s < 60) return "just now"; if (s < 3600) return `${Math.floor(s / 60)}m ago`;
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`; if (s < 604800) return `${Math.floor(s / 86400)}d ago`;
  return d.toLocaleDateString();
};
const copy = async (t: string) => {
  try { if (navigator.clipboard?.writeText) { await navigator.clipboard.writeText(t); return true; } } catch {}
  const ta = document.createElement("textarea"); ta.value = t; ta.style.position = "fixed"; ta.style.opacity = "0";
  document.body.appendChild(ta); ta.select(); const ok = document.execCommand("copy"); ta.remove(); return ok;
};

/* ─────────────── State ─────────────── */
let data: any = null;
let toast: { msg: string; ok: boolean } | null = null;
let secret: string | null = null;
let expanded = new Set<string>();
let pg = { key: "", provider: "", model: "", mode: "chat" as "chat" | "responses", running: false, messages: [] as { role: string; content: string }[], usage: null as any };
let filters = { provider: "all", status: "all", query: "", limit: "8" };
let modelSearch = "";

function setToast(msg: string, ok = true) {
  toast = { msg, ok };
  render();
  setTimeout(() => { toast = null; render(); }, 2200);
}

function toggle(id: string) { expanded.has(id) ? expanded.delete(id) : expanded.add(id); render(); }

/* ─────────────── Logo SVG ─────────────── */
const LOGO_SVG = `<svg viewBox="0 0 120 120" fill="none" class="w-full h-full">
  <defs><clipPath id="bl"><rect x="0" y="0" width="60" height="120"/></clipPath></defs>
  <circle cx="60" cy="60" r="52" stroke="currentColor" stroke-width="3" opacity="0.9"/>
  <g clip-path="url(#bl)">
    <path d="M60 14 C58 16 56 18 54 21 C52 24 50 28 48 32 C44 40 42 48 41 56 C40 64 40 72 41 80 C42 84 43 88 45 90 C47 92 49 93 52 93 C54 93 56 92 58 90 C60 88 61 84 61 80 C61 72 61 64 61 56 C61 48 61 40 61 32 C61 28 61 24 61 21 C61 18 61 16 60 14Z" fill="currentColor" opacity="0.85"/>
  </g>
  <g transform="translate(120 0) scale(-1 1)" clip-path="url(#bl)">
    <path d="M60 14 C58 16 56 18 54 21 C52 24 50 28 48 32 C44 40 42 48 41 56 C40 64 40 72 41 80 C42 84 43 88 45 90 C47 92 49 93 52 93 C54 93 56 92 58 90 C60 88 61 84 61 80 C61 72 61 64 61 56 C61 48 61 40 61 32 C61 28 61 24 61 21 C61 18 61 16 60 14Z" fill="currentColor" opacity="0.35"/>
  </g>
</svg>`;

/* ─────────────── Render ─────────────── */
function render() {
  const app = $("app");
  if (!app) return;

  const d = data;
  const apiUrl = d ? new URL(d.service.api_base_url, location.origin).toString() : "";

  app.innerHTML = `
    <div class="fixed inset-0 pointer-events-none" style="background:radial-gradient(ellipse 80% 50% at 50% 20%,rgba(250,249,246,0.025),transparent 60%)"></div>

    <nav class="fixed top-0 left-0 right-0 z-50 border-b" style="border-color:rgba(226,226,226,0.06);background:rgba(14,14,13,0.85);backdrop-filter:blur(20px)">
      <div class="mx-auto flex h-14 max-w-2xl items-center justify-between px-6">
        <div class="flex items-center gap-2.5">
          <div class="h-5 w-5 text-[var(--text)]">${LOGO_SVG}</div>
          <span class="text-[14px] font-medium tracking-tight" style="font-family:var(--font-matter)">Gunmetal</span>
        </div>
        <div class="flex items-center gap-2">
          <button id="btn-copy" class="flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-[13px] transition-[transform,background-color,border-color] duration-150 hover:bg-[rgba(255,255,255,0.04)] active:scale-[0.97]" style="border-color:rgba(226,226,226,0.1);color:var(--color-muted)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
            <span class="hidden sm:inline">Copy API</span>
          </button>
          <button id="btn-refresh" class="flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-[13px] transition-[transform,background-color,border-color] duration-150 hover:bg-[rgba(255,255,255,0.04)] active:scale-[0.97]" style="border-color:rgba(226,226,226,0.1);color:var(--color-muted)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
          </button>
        </div>
      </div>
    </nav>

    <main class="relative mx-auto flex min-h-dvh max-w-2xl flex-col items-center px-6 pt-32 pb-20">
      ${d ? renderBody(d, apiUrl) : renderLoading()}
    </main>

    ${toast ? `<div id="toast" class="fixed bottom-5 right-5 z-[100] rounded-xl border px-5 py-3 text-[13px] shadow-2xl" style="border-color:${toast.ok ? "rgba(125,211,200,0.2)" : "rgba(255,141,120,0.2)"};background:${toast.ok ? "rgba(125,211,200,0.06)" : "rgba(255,141,120,0.06)"};backdrop-filter:blur(12px);animation:toastIn 220ms cubic-bezier(0.23,1,0.32,1) both">${esc(toast.msg)}</div>` : ""}
  `;

  bind(apiUrl);
}

function renderLoading() {
  return `
    <div class="flex flex-col items-center gap-6">
      <div class="h-24 w-24 text-[var(--color-faint)]" style="animation:logoPulse 2s ease-in-out infinite">${LOGO_SVG}</div>
      <p class="text-[15px]" style="color:var(--color-muted)">Loading local state...</p>
    </div>
  `;
}

function renderBody(d: any, apiUrl: string) {
  const setup = [
    { label: "Connect provider", done: d.setup.provider_ready, n: d.counts.profiles },
    { label: "Sync models", done: d.setup.models_ready, n: d.counts.models },
    { label: "Create key", done: d.setup.key_ready, n: d.counts.keys },
    { label: "Send request", done: d.setup.traffic_ready, n: d.counts.logs },
  ];
  const allDone = setup.every((s) => s.done);
  const providers = d.profiles.filter((p: any) => d.models.some((m: any) => m.provider === p.provider));
  const models = d.models.filter((m: any) => m.provider === pg.provider);
  if (!pg.provider && providers.length) pg.provider = providers[0].provider;
  if (!pg.model && models.length) pg.model = models[0]?.id || d.models[0]?.id;

  return `
    <div class="logo-wrap relative h-20 w-20 text-[var(--text)] transition-transform duration-200" style="transition-timing-function:cubic-bezier(0.23,1,0.32,1)">
      ${LOGO_SVG}
    </div>

    <div class="mt-5 flex items-center gap-2">
      <span class="h-1.5 w-1.5 rounded-full" style="background:${d.setup.traffic_ready ? "#7dbeb3" : d.setup.provider_ready ? "var(--color-accent)" : "#ff8d78"}"></span>
      <span class="text-[15px]" style="color:var(--color-muted)">${esc(d.setup.next_step)}</span>
    </div>

    <div class="mt-8 flex items-baseline gap-6">
      ${[
        [d.counts.profiles, "Providers"],
        [d.counts.models, "Models"],
        [d.counts.keys, "Keys"],
        [d.counts.logs, "Requests"],
      ]
        .map(
          ([n, l]) => `
            <div class="flex flex-col items-center gap-1">
              <span class="font-mono text-[28px] font-medium leading-none" style="color:var(--color-text)">${n}</span>
              <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">${l}</span>
            </div>
          `
        )
        .join("")}
    </div>

    ${!allDone ? `
      <div class="mt-8 flex flex-wrap items-center justify-center gap-2">
        ${setup
          .map(
            (s, i) => `
              <div class="flex items-center gap-2">
                <div class="flex items-center gap-2 rounded-full border px-3 py-1.5 ${s.done ? "border-[rgba(125,211,200,0.25)] bg-[rgba(125,211,200,0.06)]" : "border-[rgba(226,226,226,0.08)] bg-[rgba(255,255,255,0.02)]"}">
                  <span class="flex h-4 w-4 items-center justify-center rounded-full text-[10px] font-medium ${s.done ? "bg-[#7dbeb3] text-[#0e0e0d]" : "border border-[rgba(226,226,226,0.12)] text-[var(--color-faint)]"}">${s.done ? "✓" : i + 1}</span>
                  <span class="text-[12px] ${s.done ? "text-[var(--color-muted)]" : "text-[var(--color-text)]"}">${s.label}</span>
                </div>
                ${i < 3 ? `<span class="text-[var(--color-faint)]">→</span>` : ""}
              </div>
            `
          )
          .join("")}
      </div>
    ` : ""}

    <div class="mt-10 flex flex-wrap items-center justify-center gap-2">
      ${[
        ["providers", "Providers", d.counts.profiles],
        ["activity", "Activity", d.counts.logs],
        ["playground", "Playground", null],
        ["keys", "Keys", d.counts.keys],
        ["models", "Models", d.counts.models],
      ]
        .map(
          ([id, label, count]) => `
            <button data-toggle="${id}" class="toggle-btn rounded-full border px-4 py-1.5 text-[13px] transition-[transform,background-color,border-color] duration-150 active:scale-[0.97] ${expanded.has(id) ? "border-[rgba(215,180,106,0.35)] bg-[rgba(215,180,106,0.08)] text-[var(--color-accent)]" : "border-[rgba(226,226,226,0.08)] text-[var(--color-muted)]"}">
              ${label}${count !== null ? ` <span class="font-mono text-[11px] opacity-60">${count}</span>` : ""}
            </button>
          `
        )
        .join("")}
    </div>

    <div class="mt-6 w-full space-y-4">
      ${expanded.has("providers") ? renderProviders(d) : ""}
      ${expanded.has("activity") ? renderActivity(d) : ""}
      ${expanded.has("playground") ? renderPlayground(d, providers, models) : ""}
      ${expanded.has("keys") ? renderKeys(d) : ""}
      ${expanded.has("models") ? renderModels(d) : ""}
    </div>

    ${secret ? `
      <div class="mt-6 w-full rounded-xl border px-4 py-3" style="border-color:rgba(215,180,106,0.25);background:rgba(215,180,106,0.06);animation:fadeUp 250ms cubic-bezier(0.23,1,0.32,1) both">
        <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-accent)">New key secret</span>
        <code class="mt-1 block break-all font-mono text-[13px]" style="color:var(--color-text)">${esc(secret)}</code>
        <p class="mt-1 text-[12px]" style="color:var(--color-muted)">Copy this now. It will not be shown again.</p>
      </div>
    ` : ""}

    <div class="mt-auto pt-16 flex flex-col items-center gap-1 text-[12px]" style="color:var(--color-faint)">
      <span class="font-mono">${esc(apiUrl)}</span>
      <span>v${esc(d.service.version)} · ${esc(d.service.home)}</span>
    </div>
  `;
}

function renderProviders(d: any) {
  return `
    <div class="w-full" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <div class="flex items-center justify-between mb-3">
        <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">Providers</span>
        <button id="btn-add-provider" class="text-[13px] transition-colors duration-150 hover:text-[var(--color-text)]" style="color:var(--color-accent)">+ Add provider</button>
      </div>
      ${d.profiles.length === 0 ? `<p class="text-[14px] text-center py-8" style="color:var(--color-muted)">No providers connected. Add one to start.</p>` : `
        <div class="space-y-2">
          ${d.profiles
            .map(
              (p: any, i: number) => `
                <div class="provider-row flex items-center justify-between rounded-xl border px-4 py-3 transition-[background-color] duration-150" style="border-color:rgba(226,226,226,0.06);animation:fadeUp 200ms cubic-bezier(0.23,1,0.32,1) both;animation-delay:${i * 40}ms">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="text-[14px]">${esc(p.name)}</span>
                      <span class="rounded-full border px-2 py-0.5 text-[10px]" style="border-color:rgba(226,226,226,0.08);color:var(--color-faint)">${esc(p.provider)}</span>
                    </div>
                    <div class="mt-0.5 font-mono text-[12px]" style="color:var(--color-faint)">${esc(p.selector)} · ${p.model_count} models</div>
                  </div>
                  <div class="flex gap-1">
                    <button class="p-act rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)" data-id="${p.id}" data-a="auth">Auth</button>
                    <button class="p-act rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)" data-id="${p.id}" data-a="sync">Sync</button>
                    <button class="p-key rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)" data-id="${p.id}">Key</button>
                    <button class="p-del rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(255,141,120,0.15);color:var(--color-muted)" data-id="${p.id}">×</button>
                  </div>
                </div>
              `
            )
            .join("")}
        </div>
      `}
      ${renderAddProviderForm(d)}
    </div>
  `;
}

function renderAddProviderForm(d: any) {
  return `
    <form id="form-provider" class="mt-3 hidden" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <div class="rounded-xl border p-4 space-y-3" style="border-color:rgba(226,226,226,0.1);background:rgba(255,255,255,0.02)">
        <select name="provider" id="sel-provider" class="h-9 w-full rounded-lg border bg-transparent px-3 text-[13px] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)">
          ${d.providers.map((p: any) => `<option value="${esc(p.kind)}">${esc(p.kind)}</option>`).join("")}
        </select>
        <input name="name" required placeholder="Provider name" class="h-9 w-full rounded-lg border bg-transparent px-3 text-[13px] placeholder:text-[var(--color-faint)] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)" />
        <input name="base_url" placeholder="Base URL (optional)" class="h-9 w-full rounded-lg border bg-transparent px-3 text-[13px] placeholder:text-[var(--color-faint)] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)" />
        <input name="api_key" type="password" placeholder="API key" class="h-9 w-full rounded-lg border bg-transparent px-3 text-[13px] placeholder:text-[var(--color-faint)] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)" />
        <div class="flex gap-2">
          <button type="submit" class="rounded-lg border px-4 py-2 text-[13px] transition-[transform,background-color] duration-150 hover:bg-[rgba(215,180,106,0.12)] active:scale-[0.97]" style="border-color:rgba(215,180,106,0.3);color:var(--color-accent)">Save</button>
          <button type="button" id="btn-cancel-provider" class="rounded-lg border px-4 py-2 text-[13px] transition-[transform,background-color] duration-150 hover:bg-[rgba(255,255,255,0.04)] active:scale-[0.97]" style="border-color:rgba(226,226,226,0.1);color:var(--color-muted)">Cancel</button>
        </div>
      </div>
    </form>
  `;
}

function renderActivity(d: any) {
  const f = filters;
  const filtered = d.logs.filter((log: any) => {
    const pm = f.provider === "all" || `${log.provider}::${log.profile_name || ""}` === f.provider;
    const sm = f.status === "all" || (f.status === "success" ? (log.status_code ?? 0) < 400 && !log.error_message : (log.status_code ?? 0) >= 400 || !!log.error_message);
    const q = f.query.trim().toLowerCase();
    const qm = !q || [log.provider, log.profile_name || "", log.model, log.endpoint, log.key_name || "", log.request_mode || "", log.error_message || ""].join(" ").toLowerCase().includes(q);
    return pm && sm && qm;
  });
  const visible = f.limit === "all" ? filtered : filtered.slice(0, Number(f.limit) || 8);

  return `
    <div class="w-full" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <div class="flex flex-wrap items-center gap-2 mb-3">
        <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">Activity</span>
        <select id="filt-p" class="h-7 rounded-md border bg-transparent px-2 text-[12px]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)">
          <option value="all">All providers</option>
          ${d.provider_summaries.map((s: any) => `<option value="${esc(`${s.provider}::${s.profile_name || ""}`)}"${f.provider === `${s.provider}::${s.profile_name || ""}` ? " selected" : ""}>${esc(s.label)}</option>`).join("")}
        </select>
        <select id="filt-s" class="h-7 rounded-md border bg-transparent px-2 text-[12px]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)">
          ${["all","success","error"].map((v) => `<option value="${v}"${f.status === v ? " selected" : ""}>${v}</option>`).join("")}
        </select>
        <input id="filt-q" value="${esc(f.query)}" placeholder="Search..." class="h-7 rounded-md border bg-transparent px-2 text-[12px] placeholder:text-[var(--color-faint)]" style="border-color:rgba(226,226,226,0.08);color:var(--color-text)" />
        <select id="filt-l" class="h-7 rounded-md border bg-transparent px-2 text-[12px]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)">
          ${["8","24","all"].map((v) => `<option value="${v}"${f.limit === v ? " selected" : ""}>${v === "all" ? "All" : `Latest ${v}`}</option>`).join("")}
        </select>
      </div>
      ${visible.length === 0 ? `<p class="text-[14px] text-center py-8" style="color:var(--color-muted)">No activity to show.</p>` : `
        <div class="space-y-1">
          ${visible
            .map(
              (log: any, i: number) => `
                <div class="log-row flex items-center gap-3 rounded-xl border px-4 py-2.5 transition-[background-color] duration-150" style="border-color:rgba(226,226,226,0.04);animation:fadeUp 180ms cubic-bezier(0.23,1,0.32,1) both;animation-delay:${i * 30}ms">
                  <span class="h-1.5 w-1.5 shrink-0 rounded-full" style="background:${(log.status_code ?? 0) >= 400 || log.error_message ? "#ff8d78" : "#7dbeb3"}"></span>
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-[13px]">${esc(log.model)}</div>
                    <div class="text-[11px]" style="color:var(--color-faint)">${esc(log.profile_name || log.provider)} · ${esc(log.request_mode || log.endpoint)} · ${fmtRel(log.started_at)}</div>
                  </div>
                  <div class="shrink-0 text-right">
                    <div class="font-mono text-[12px]" style="color:var(--color-muted)">${log.duration_ms}ms</div>
                    ${log.total_tokens ? `<div class="text-[11px]" style="color:var(--color-faint)">${log.total_tokens} tok</div>` : ""}
                  </div>
                </div>
              `
            )
            .join("")}
        </div>
        <p class="mt-2 text-[11px] text-center" style="color:var(--color-faint)">${visible.length} of ${filtered.length} matching</p>
      `}
    </div>
  `;
}

function renderPlayground(d: any, providers: any[], models: any[]) {
  return `
    <div class="w-full" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">Playground</span>
      <div class="mt-3 grid gap-2 sm:grid-cols-4">
        <input id="pg-key" value="${esc(pg.key)}" placeholder="gm_..." class="h-9 rounded-lg border bg-transparent px-3 font-mono text-[12px] placeholder:text-[var(--color-faint)] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)" />
        <select id="pg-p" class="h-9 rounded-lg border bg-transparent px-3 text-[13px] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)">
          ${providers.length ? providers.map((p: any) => `<option value="${esc(p.provider)}"${pg.provider === p.provider ? " selected" : ""}>${esc(p.name)}</option>`).join("") : `<option>No providers</option>`}
        </select>
        <select id="pg-m" class="h-9 rounded-lg border bg-transparent px-3 font-mono text-[11px] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)">
          ${models.length ? models.map((m: any) => `<option value="${esc(m.id)}"${pg.model === m.id ? " selected" : ""}>${esc(m.id)}</option>`).join("") : `<option>No models</option>`}
        </select>
        <select id="pg-mode" class="h-9 rounded-lg border bg-transparent px-3 text-[13px] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)">
          <option value="chat"${pg.mode === "chat" ? " selected" : ""}>chat/completions</option>
          <option value="responses"${pg.mode === "responses" ? " selected" : ""}>responses</option>
        </select>
      </div>
      <div class="mt-3 max-h-[320px] overflow-auto rounded-xl border p-3 space-y-3" style="border-color:rgba(226,226,226,0.06)">
        ${pg.messages.length
          ? pg.messages
              .map(
                (m) => `
                  <div>
                    <span class="text-[10px] font-medium uppercase tracking-[0.12em] ${m.role === "user" ? "text-[var(--color-accent)]" : "text-[#7dbeb3]"}">${m.role}</span>
                    <p class="mt-1 whitespace-pre-wrap text-[14px] leading-relaxed">${esc(m.content)}</p>
                  </div>
                `
              )
              .join("")
          : `<p class="text-center py-8 text-[13px]" style="color:var(--color-muted)">Choose a model and send a message</p>`}
      </div>
      <form id="pg-form" class="mt-3 flex gap-2">
        <textarea id="pg-in" placeholder="Ask something..." rows="2" class="min-h-[44px] flex-1 resize-y rounded-lg border bg-transparent px-3 py-2 text-[14px] placeholder:text-[var(--color-faint)] focus:border-[rgba(215,180,106,0.4)]" style="border-color:rgba(226,226,226,0.1);color:var(--color-text)" ${pg.running ? "disabled" : ""}></textarea>
        <button type="submit" class="self-end rounded-lg border px-4 py-2 text-[13px] transition-[transform,background-color] duration-150 hover:bg-[rgba(215,180,106,0.1)] active:scale-[0.97]" style="border-color:rgba(215,180,106,0.25);color:var(--color-accent)" ${pg.running ? "disabled" : ""}>${pg.running ? "..." : "Send"}</button>
        <button type="button" id="pg-clr" class="self-end rounded-lg border px-3 py-2 text-[13px] transition-[transform,background-color] duration-150 hover:bg-[rgba(255,255,255,0.04)] active:scale-[0.97]" style="border-color:rgba(226,226,226,0.1);color:var(--color-muted)" ${pg.running || !pg.messages.length ? "disabled" : ""}>Clear</button>
      </form>
    </div>
  `;
}

function renderKeys(d: any) {
  return `
    <div class="w-full" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">Keys</span>
      ${d.keys.length === 0 ? `<p class="text-[14px] text-center py-8" style="color:var(--color-muted)">No keys yet.</p>` : `
        <div class="mt-3 space-y-2">
          ${d.keys
            .map(
              (k: any, i: number) => `
                <div class="key-row flex items-center justify-between rounded-xl border px-4 py-2.5 transition-[background-color] duration-150" style="border-color:rgba(226,226,226,0.04);animation:fadeUp 180ms cubic-bezier(0.23,1,0.32,1) both;animation-delay:${i * 40}ms">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="text-[14px]">${esc(k.name)}</span>
                      <span class="rounded-full border px-2 py-0.5 text-[10px] ${k.state === "active" ? "border-[rgba(125,211,200,0.2)] bg-[rgba(125,211,200,0.06)]" : "border-[rgba(226,226,226,0.06)] bg-[rgba(255,255,255,0.02)]"}" style="color:var(--color-muted)">${k.state}</span>
                    </div>
                    <div class="font-mono text-[12px]" style="color:var(--color-faint)">${esc(k.prefix)} · ${k.providers.length ? esc(k.providers.join(", ")) : "all"}</div>
                  </div>
                  <div class="flex gap-1">
                    <button class="k-act rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)" data-id="${k.id}" data-s="disabled">Disable</button>
                    <button class="k-act rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(226,226,226,0.08);color:var(--color-muted)" data-id="${k.id}" data-s="revoked">Revoke</button>
                    <button class="k-del rounded-md border px-2 py-1 text-[11px] transition-[transform,background-color,border-color] duration-120 active:scale-[0.97]" style="border-color:rgba(255,141,120,0.12);color:var(--color-muted)" data-id="${k.id}">×</button>
                  </div>
                </div>
              `
            )
            .join("")}
        </div>
      `}
    </div>
  `;
}

function renderModels(d: any) {
  const q = modelSearch.trim().toLowerCase();
  const filtered = q ? d.models.filter((m: any) => m.id.toLowerCase().includes(q) || m.provider.toLowerCase().includes(q)) : d.models;
  return `
    <div class="w-full" style="animation:fadeUp 220ms cubic-bezier(0.23,1,0.32,1) both">
      <div class="flex items-center justify-between mb-3">
        <span class="text-[11px] font-medium uppercase tracking-[0.12em]" style="color:var(--color-faint)">Models</span>
        <input id="mod-q" value="${esc(modelSearch)}" placeholder="Search..." class="h-7 w-40 rounded-md border bg-transparent px-2 text-[12px] placeholder:text-[var(--color-faint)]" style="border-color:rgba(226,226,226,0.08);color:var(--color-text)" />
      </div>
      ${filtered.length === 0 ? `<p class="text-[14px] text-center py-8" style="color:var(--color-muted)">No models found.</p>` : `
        <div class="grid gap-1.5 sm:grid-cols-2">
          ${filtered.slice(0, 40).map((m: any, i: number) => `
            <div class="model-row flex items-center justify-between rounded-lg border px-3 py-2 transition-[background-color] duration-150" style="border-color:rgba(226,226,226,0.04);animation:fadeUp 160ms cubic-bezier(0.23,1,0.32,1) both;animation-delay:${i * 25}ms">
              <div class="min-w-0">
                <div class="truncate font-mono text-[12px]">${esc(m.id)}</div>
                <div class="text-[10px]" style="color:var(--color-faint)">${esc(m.provider)}</div>
              </div>
              <div class="flex gap-1 shrink-0">
                ${m.supports_reasoning ? `<span class="rounded-full border px-1.5 py-0.5 text-[9px]" style="border-color:rgba(226,226,226,0.06);color:var(--color-faint)">reason</span>` : ""}
                ${m.supports_tools ? `<span class="rounded-full border px-1.5 py-0.5 text-[9px]" style="border-color:rgba(226,226,226,0.06);color:var(--color-faint)">tools</span>` : ""}
              </div>
            </div>
          `).join("")}
        </div>
      `}
    </div>
  `;
}

/* ─────────────── Bindings ─────────────── */
function bind(apiUrl: string) {
  $("btn-copy")?.addEventListener("click", async () => {
    const ok = await copy(apiUrl);
    setToast(ok ? "Copied API URL" : "Copy failed", ok);
  });

  $("btn-refresh")?.addEventListener("click", () => { load(); });

  document.querySelectorAll("[data-toggle]").forEach((el) => {
    el.addEventListener("click", () => toggle(el.getAttribute("data-toggle")!));
  });

  document.querySelectorAll(".p-act").forEach((el) => {
    el.addEventListener("click", async () => {
      const id = (el as HTMLElement).dataset.id!;
      const a = (el as HTMLElement).dataset.a!;
      try { const r = await api.action(id, a); if (r.auth_url) window.open(r.auth_url, "_blank", "noopener,noreferrer"); setToast(r.message); load(); } catch (e: any) { setToast(e.message, false); }
    });
  });
  document.querySelectorAll(".p-key").forEach((el) => {
    el.addEventListener("click", async () => {
      const id = (el as HTMLElement).dataset.id!;
      const name = window.prompt("Key name", "gunmetal-key");
      if (name == null) return;
      try { const r = await api.createKey(id, name); setToast(r.message); secret = r.secret || null; if (secret) pg.key = secret; load(); } catch (e: any) { setToast(e.message, false); }
    });
  });
  document.querySelectorAll(".p-del").forEach((el) => {
    el.addEventListener("click", async () => {
      const id = (el as HTMLElement).dataset.id!;
      if (!window.confirm("Delete provider and its models?")) return;
      try { const r = await api.deleteProfile(id); setToast(r.message); load(); } catch (e: any) { setToast(e.message, false); }
    });
  });

  $("btn-add-provider")?.addEventListener("click", () => {
    const f = $("form-provider"); if (f) f.classList.toggle("hidden");
  });
  $("btn-cancel-provider")?.addEventListener("click", () => {
    const f = $("form-provider"); if (f) { f.classList.add("hidden"); (f as HTMLFormElement).reset(); }
  });
  $("form-provider")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const fd = new FormData(e.target as HTMLFormElement);
    try {
      const r = await api.createProfile({ provider: String(fd.get("provider")), name: String(fd.get("name")), base_url: String(fd.get("base_url")), api_key: String(fd.get("api_key")) });
      setToast(r.message);
      $("form-provider")?.classList.add("hidden");
      (e.target as HTMLFormElement).reset();
      load();
    } catch (e: any) { setToast(e.message, false); }
  });

  document.querySelectorAll(".k-act").forEach((el) => {
    el.addEventListener("click", async () => {
      try { const r = await api.setKeyState((el as HTMLElement).dataset.id!, (el as HTMLElement).dataset.s!); setToast(r.message); load(); } catch (e: any) { setToast(e.message, false); }
    });
  });
  document.querySelectorAll(".k-del").forEach((el) => {
    el.addEventListener("click", async () => {
      if (!window.confirm("Delete key?")) return;
      try { const r = await api.deleteKey((el as HTMLElement).dataset.id!); setToast(r.message); load(); } catch (e: any) { setToast(e.message, false); }
    });
  });

  $("filt-p")?.addEventListener("change", (e) => { filters.provider = (e.target as HTMLSelectElement).value; render(); });
  $("filt-s")?.addEventListener("change", (e) => { filters.status = (e.target as HTMLSelectElement).value; render(); });
  $("filt-q")?.addEventListener("input", (e) => { filters.query = (e.target as HTMLInputElement).value; render(); });
  $("filt-l")?.addEventListener("change", (e) => { filters.limit = (e.target as HTMLSelectElement).value; render(); });

  $("mod-q")?.addEventListener("input", (e) => { modelSearch = (e.target as HTMLInputElement).value; render(); });

  $("pg-key")?.addEventListener("input", (e) => { pg.key = (e.target as HTMLInputElement).value; });
  $("pg-p")?.addEventListener("change", (e) => {
    pg.provider = (e.target as HTMLSelectElement).value;
    if (data) { const ms = data.models.filter((m: any) => m.provider === pg.provider); pg.model = ms[0]?.id || data.models[0]?.id; }
    render();
  });
  $("pg-m")?.addEventListener("change", (e) => { pg.model = (e.target as HTMLSelectElement).value; });
  $("pg-mode")?.addEventListener("change", (e) => { pg.mode = (e.target as HTMLSelectElement).value as "chat" | "responses"; });
  $("pg-clr")?.addEventListener("click", () => { pg.messages = []; pg.usage = null; render(); });
  $("pg-form")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const input = $("pg-in") as HTMLTextAreaElement;
    const content = input?.value.trim() || "";
    if (!pg.key) { setToast("Paste a Gunmetal key", false); return; }
    if (!pg.model) { setToast("Sync a model first", false); return; }
    if (!content) { setToast("Enter a message", false); return; }
    pg.messages.push({ role: "user", content }); pg.running = true; pg.usage = null;
    if (input) input.value = ""; render();
    try {
      const res = await api.chat(pg.mode, pg.key, pg.model, pg.messages);
      if (!res.ok) { const t = await res.text(); let m = "request failed"; try { m = JSON.parse(t)?.error?.message || m; } catch {} throw new Error(m); }
      pg.messages.push({ role: "assistant", content: "" }); render();
      const reader = res.body?.getReader(); const decoder = new TextDecoder(); if (!reader) throw new Error("No stream");
      let buf = "", text = "";
      while (true) {
        const { value, done } = await reader.read();
        buf += decoder.decode(value || new Uint8Array(), { stream: !done });
        let b = buf.indexOf("\n\n");
        while (b !== -1) {
          const chunk = buf.slice(0, b); buf = buf.slice(b + 2);
          const lines = chunk.split(/\r?\n/); let ev = "message"; const dl: string[] = [];
          for (const line of lines) { if (line.startsWith("event:")) ev = line.slice(6).trim(); else if (line.startsWith("data:")) dl.push(line.slice(5).trimStart()); }
          if (dl.length && dl.join("") !== "[DONE]") {
            const parsed = JSON.parse(dl.join("\n"));
            if (pg.mode === "chat") { const d = parsed?.choices?.[0]?.delta?.content || ""; if (d) text += d; }
            else if (ev === "response.output_text.delta") text += parsed.delta || "";
            else if (ev === "response.completed") { text = parsed?.response?.output_text || text; pg.usage = parsed?.response?.usage || null; }
            const last = pg.messages[pg.messages.length - 1]; if (last && last.role === "assistant") last.content = text;
            render();
          }
          b = buf.indexOf("\n\n");
        }
        if (done) break;
      }
      setToast("Done");
    } catch (err: any) { pg.messages = pg.messages.slice(0, Math.max(0, pg.messages.length - 2)); setToast(err.message, false); }
    finally { pg.running = false; render(); }
  });

  document.addEventListener("keydown", (e) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (e.key === "r" && !e.metaKey && !e.ctrlKey) { e.preventDefault(); load(); }
    if (e.key === "c" && !e.metaKey && !e.ctrlKey) { e.preventDefault(); ($("pg-in") as HTMLTextAreaElement)?.focus(); }
    if (e.key === "Escape") { expanded.clear(); render(); }
  });
}

/* ─────────────── Init ─────────────── */
async function load() {
  try {
    data = await api.state();
    if (!pg.provider && data.profiles.length) pg.provider = data.profiles[0].provider;
    if (!pg.model && data.models.length) { const ms = data.models.filter((m: any) => m.provider === pg.provider); pg.model = ms[0]?.id || data.models[0].id; }
  } catch { data = null; }
  render();
}

load();
