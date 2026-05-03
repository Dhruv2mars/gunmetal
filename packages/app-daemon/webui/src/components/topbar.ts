import { escapeHtml, copyToClipboard } from "../utils";
import { copyApiUrl } from "../api";
import { uiState, setToast, render } from "../state";

export function renderTopbar(): string {
  const data = uiState.data;
  const apiBase = data ? new URL(data.service.api_base_url, window.location.origin).toString() : "";

  return `
    <nav class="sticky top-0 z-50 border-b hairline" style="background:rgba(14,14,13,0.88);backdrop-filter:blur(20px)">
      <div class="mx-auto flex h-14 max-w-5xl items-center justify-between px-4 md:px-6">
        <div class="flex items-center gap-2.5">
          <span class="flex h-7 w-7 items-center justify-center rounded-full border hairline-strong text-[13px] font-mono font-medium">
            G
          </span>
          <div class="flex flex-col leading-none">
            <span class="text-[14px] font-medium tracking-[-0.01em]">Gunmetal</span>
            <span class="text-[11px] text-text-muted">Local dashboard</span>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <div class="hidden items-center gap-2 md:flex">
            <span class="h-1.5 w-1.5 rounded-full" style="background:#7dbeb3"></span>
            <span class="text-[12px] text-text-muted">Running</span>
            <span class="mx-1 text-text-faint">·</span>
            <code class="font-mono text-[12px] text-text-muted">${escapeHtml(apiBase)}</code>
          </div>
          <button id="copy-api-btn" class="tap-target flex items-center gap-1.5 rounded-lg border hairline px-3 py-1.5 text-[13px] text-text-secondary transition-all duration-150 hover:bg-frosted-hover active:scale-[0.97]">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            Copy API
          </button>
          <button id="refresh-btn" class="tap-target flex items-center gap-1.5 rounded-lg border hairline px-3 py-1.5 text-[13px] text-text-secondary transition-all duration-150 hover:bg-frosted-hover active:scale-[0.97]">
            <svg id="refresh-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
            Refresh
          </button>
        </div>
      </div>
    </nav>
  `;
}

export function bindTopbar() {
  document.getElementById("copy-api-btn")?.addEventListener("click", async () => {
    const ok = await copyApiUrl();
    setToast(ok ? "Copied API base URL" : "Copy failed", ok ? "success" : "danger");
  });

  document.getElementById("refresh-btn")?.addEventListener("click", () => {
    const icon = document.getElementById("refresh-icon");
    if (icon) {
      icon.style.transition = "transform 400ms linear";
      icon.style.transform = "rotate(360deg)";
      setTimeout(() => {
        icon.style.transition = "none";
        icon.style.transform = "rotate(0deg)";
      }, 400);
    }
    window.dispatchEvent(new CustomEvent("gm:refresh"));
  });
}
