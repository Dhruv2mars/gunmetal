import { escapeHtml } from "../utils";
import { setKeyState, deleteKey } from "../api";
import { uiState, setToast, render } from "../state";

export function renderKeys(): string {
  const data = uiState.data;
  if (!data) return "";

  return `
    <section class="mx-auto max-w-5xl border-b hairline px-4 py-6 md:px-6">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-[18px] font-medium tracking-[-0.02em] text-text">Keys</h2>
          <p class="mt-0.5 text-[13px] text-text-muted">Gunmetal keys for your apps</p>
        </div>
      </div>

      ${data.keys.length === 0 ? renderEmpty() : renderList(data.keys)}

      ${uiState.secret ? `
        <div class="mt-4 animate-fade-in rounded-xl border border-accent/30 bg-accent-muted px-4 py-3">
          <span class="text-label text-accent">New key secret</span>
          <code class="mt-1 block break-all font-mono text-[13px] text-text">${escapeHtml(uiState.secret)}</code>
          <p class="mt-1 text-[12px] text-text-muted">Copy this now. It will not be shown again.</p>
        </div>
      ` : ""}
    </section>
  `;
}

function renderEmpty(): string {
  return `
    <div class="mt-4 flex flex-col items-center justify-center rounded-xl border border-dashed hairline py-12">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-text-faint"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
      <p class="mt-2 text-[14px] text-text-muted">No keys yet</p>
      <p class="mt-0.5 text-[13px] text-text-faint">Create a key from a provider to start using the API</p>
    </div>
  `;
}

function renderList(keys: typeof uiState.data.keys): string {
  return `
    <div class="mt-4 divide-y hairline">
      ${keys
        .map((key) => {
          const stateColor =
            key.state === "active"
              ? "border-success-border bg-success-bg text-text-secondary"
              : key.state === "disabled"
              ? "hairline bg-frosted text-text-muted"
              : "border-danger-border bg-danger-bg text-text-muted";
          return `
            <div class="flex flex-col gap-2 py-3 sm:flex-row sm:items-center sm:justify-between">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-[15px] font-medium text-text">${escapeHtml(key.name)}</span>
                  <span class="rounded-full border px-2 py-0.5 text-[11px] ${stateColor}">${escapeHtml(key.state)}</span>
                </div>
                <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-[12px] text-text-faint">
                  <span class="font-mono">${escapeHtml(key.prefix)}</span>
                  <span>${key.providers.length ? escapeHtml(key.providers.join(", ")) : "all providers"}</span>
                  ${key.last_used_at ? `<span>used ${escapeHtml(new Date(key.last_used_at).toLocaleDateString())}</span>` : ""}
                </div>
              </div>
              <div class="flex flex-wrap gap-1.5">
                <button class="key-action tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-action="disabled" data-id="${escapeHtml(key.id)}">Disable</button>
                <button class="key-action tap-target rounded-md border hairline px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" data-action="revoked" data-id="${escapeHtml(key.id)}">Revoke</button>
                <button class="key-delete tap-target rounded-md border border-danger-border bg-danger-bg px-2.5 py-1 text-[12px] text-text-secondary transition-all hover:opacity-80 active:scale-[0.97]" data-id="${escapeHtml(key.id)}">Delete</button>
              </div>
            </div>
          `;
        })
        .join("")}
    </div>
  `;
}

export function bindKeys() {
  document.querySelectorAll(".key-action").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-id")!;
      const action = btn.getAttribute("data-action")!;
      try {
        const res = await setKeyState(id, action);
        setToast(res.message, "success");
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  });

  document.querySelectorAll(".key-delete").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.getAttribute("data-id")!;
      if (!window.confirm("Delete this key?")) return;
      try {
        const res = await deleteKey(id);
        setToast(res.message, "success");
        window.dispatchEvent(new CustomEvent("gm:refresh"));
      } catch (err: any) {
        setToast(err.message, "danger");
      }
    });
  });
}
