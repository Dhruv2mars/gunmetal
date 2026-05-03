import { escapeHtml } from "../utils";
import { createProfile, profileAction, createKey, deleteProfile } from "../api";
import { uiState, setToast, setSecret, render } from "../state";

export function renderProviders(): string {
  const data = uiState.data;
  if (!data) return "";

  return `
    <section class="mx-auto max-w-5xl border-b hairline px-4 py-6 md:px-6">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-[18px] font-medium tracking-[-0.02em] text-text">Providers</h2>
          <p class="mt-0.5 text-[13px] text-text-muted">Connected upstream providers</p>
        </div>
        <button id="toggle-add-provider" class="tap-target rounded-lg border hairline px-3 py-1.5 text-[13px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]">
          ${uiState.showAddProvider ? "Cancel" : "Add provider"}
        </button>
      </div>

      ${data.profiles.length === 0 ? renderEmpty() : renderList(data.profiles)}
      ${uiState.showAddProvider ? renderAddForm(data.providers) : ""}
    </section>
  `;
}

function renderEmpty(): string {
  return `
    <div class="mt-4 flex flex-col items-center justify-center rounded-xl border border-dashed hairline py-12">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-text-faint"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
      <p class="mt-2 text-[14px] text-text-muted">No providers connected</p>
      <p class="mt-0.5 text-[13px] text-text-faint">Add your first provider to start routing requests</p>
    </div>
  `;
}

function renderList(profiles: typeof uiState.data.profiles): string {
  return `
    <div class="mt-4 divide-y hairline">
      ${profiles
        .map((profile) => {
          const provider = uiState.data?.providers.find((p) => p.kind === profile.provider);
          const modes = provider
            ? [
                provider.supports_chat_completions ? "chat" : null,
                provider.supports_responses_api ? "responses" : null,
              ]
                .filter(Boolean)
                .join(" + ") || "request"
            : "";
          const isAuthed = profile.auth_label.includes("saved") || profile.auth_label.includes("session");

          return `
            <div class="flex flex-col gap-2 py-3 sm:flex-row sm:items-center sm:justify-between">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-[15px] font-medium text-text">${escapeHtml(profile.name)}</span>
                  <span class="rounded-full border hairline px-2 py-0.5 text-[11px] text-text-muted">${escapeHtml(profile.provider)}</span>
                  <span class="h-1.5 w-1.5 rounded-full ${isAuthed ? "bg-[#7dbeb3]" : "bg-[#ff8d78]"}"></span>
                </div>
                <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-[12px] text-text-faint">
                  <span class="font-mono">${escapeHtml(profile.selector)}</span>
                  ${profile.base_url ? `<span>${escapeHtml(profile.base_url)}</span>` : ""}
                  <span>${escapeHtml(profile.auth_label)}</span>
                  <span>${profile.model_count} models</span>
                  ${modes ? `<span>${escapeHtml(modes)}</span>` : ""}
                </div>
              </div>
              <div class="flex flex-wrap gap-1.5">
                <button class="provider-action tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-action="auth" data-id="${escapeHtml(profile.id)}">Auth</button>
                <button class="provider-action tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-action="sync" data-id="${escapeHtml(profile.id)}">Sync</button>
                <button class="provider-create-key tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-id="${escapeHtml(profile.id)}">Key</button>
                <button class="provider-action tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-action="logout" data-id="${escapeHtml(profile.id)}">Logout</button>
                <button class="provider-delete tap-target rounded-md border border-danger-border bg-danger-bg px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:opacity-80 active:scale-[0.97]" data-id="${escapeHtml(profile.id)}">Delete</button>
              </div>
            </div>
          `;
        })
        .join("")}
    </div>
  `;
}

function renderAddForm(providers: typeof uiState.data.providers): string {
  const first = providers[0];
  return `
    <form id="add-provider-form" class="mt-4 animate-fade-in rounded-xl border hairline-strong bg-frosted p-4">
      <div class="grid gap-3 sm:grid-cols-2">
        <div>
          <label class="text-label text-text-faint">Provider type</label>
          <select name="provider" id="add-provider-select" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">
            ${providers.map((p) => `<option value="${escapeHtml(p.kind)}">${escapeHtml(p.kind)}</option>`).join("")}
          </select>
        </div>
        <div>
          <label class="text-label text-text-faint">Name</label>
          <input name="name" required placeholder="${escapeHtml(first?.suggested_name || "provider")}" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2.5 text-[13px] text-text placeholder:text-text-faint focus:border-accent" />
        </div>
        <div id="add-base-url-wrap">
          <label class="text-label text-text-faint">Base URL</label>
          <input name="base_url" placeholder="${escapeHtml(first?.base_url_placeholder || "optional")}" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2.5 text-[13px] text-text placeholder:text-text-faint focus:border-accent" />
        </div>
        <div id="add-api-key-wrap">
          <label class="text-label text-text-faint">API key</label>
          <input name="api_key" type="password" placeholder="required for this provider" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2.5 text-[13px] text-text placeholder:text-text-faint focus:border-accent" />
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button type="submit" class="tap-target rounded-lg border border-accent/40 bg-accent-soft px-4 py-2 text-[13px] font-medium text-accent transition-all hover:bg-accent/20 active:scale-[0.97]">Save provider</button>
      </div>
      <p id="add-provider-help" class="mt-2 text-[12px] text-text-muted">${escapeHtml(first?.helper_body || "")}</p>
    </form>
  `;
}

export function bindProviders() {
  document.getElementById("toggle-add-provider")?.addEventListener("click", () => {
    uiState.showAddProvider = !uiState.showAddProvider;
    render();
  });

  document.querySelectorAll(".provider-action").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-id")!;
      const action = btn.getAttribute("data-action")!;
      try {
        const res = await profileAction(id, action);
        if (res.auth_url) window.open(res.auth_url, "_blank", "noopener,noreferrer");
        setToast(res.message, "success");
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  });

  document.querySelectorAll(".provider-create-key").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-id")!;
      const name = window.prompt("Key name", "gunmetal-key");
      if (name == null) return;
      try {
        const res = await createKey(id, name);
        setToast(res.message, "success");
        setSecret(res.secret || null);
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  });

  document.querySelectorAll(".provider-delete").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-id")!;
      if (!window.confirm("Delete this provider and its synced models?")) return;
      try {
        const res = await deleteProfile(id);
        setToast(res.message, "success");
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  });

  const form = document.getElementById("add-provider-form") as HTMLFormElement | null;
  if (form) {
    const syncForm = () => {
      const select = document.getElementById("add-provider-select") as HTMLSelectElement;
      const kind = select?.value || "";
      const def = uiState.data?.providers.find((p) => p.kind === kind);
      const baseWrap = document.getElementById("add-base-url-wrap");
      const keyWrap = document.getElementById("add-api-key-wrap");
      const help = document.getElementById("add-provider-help");
      if (baseWrap) baseWrap.style.display = def?.supports_base_url ? "" : "none";
      if (keyWrap) keyWrap.style.display = def?.auth_method === "api_key" ? "" : "none";
      if (help && def) help.textContent = def.helper_body;
    };
    document.getElementById("add-provider-select")?.addEventListener("change", syncForm);
    syncForm();

    form.addEventListener("submit", async (e) => {
      e.preventDefault();
      const fd = new FormData(form);
      try {
        const res = await createProfile({
          provider: String(fd.get("provider") || ""),
          name: String(fd.get("name") || ""),
          base_url: String(fd.get("base_url") || ""),
          api_key: String(fd.get("api_key") || ""),
        });
        setToast(res.message, "success");
        uiState.showAddProvider = false;
        form.reset();
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  }
}
