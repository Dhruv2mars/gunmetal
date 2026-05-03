import { escapeHtml } from "../utils";
import { uiState } from "../state";

export function renderModels(): string {
  const data = uiState.data;
  if (!data) return "";

  const query = uiState.modelSearch.trim().toLowerCase();
  const filtered = query
    ? data.models.filter((m) => m.id.toLowerCase().includes(query) || m.provider.toLowerCase().includes(query))
    : data.models;

  return `
    <section class="mx-auto max-w-5xl px-4 py-6 md:px-6">
      <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 class="text-[18px] font-medium tracking-[-0.02em] text-text">Models</h2>
          <p class="mt-0.5 text-[13px] text-text-muted">Synced provider-qualified models</p>
        </div>
        <input id="model-search" value="${escapeHtml(uiState.modelSearch)}" placeholder="Search models..." class="h-8 w-full rounded-md border hairline bg-bg px-2.5 text-[13px] text-text placeholder:text-text-faint focus:border-accent sm:w-64" />
      </div>

      ${filtered.length === 0 ? renderEmpty() : renderList(filtered)}
    </section>
  `;
}

function renderEmpty(): string {
  const data = uiState.data;
  const hasModels = data && data.models.length > 0;
  return `
    <div class="mt-4 flex flex-col items-center justify-center rounded-xl border border-dashed hairline py-12">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-text-faint"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
      <p class="mt-2 text-[14px] text-text-muted">${hasModels ? "No models match your search." : "No models synced yet"}</p>
      ${!hasModels ? `<p class="mt-0.5 text-[13px] text-text-faint">Sync a provider to pull its available models</p>` : ""}
    </div>
  `;
}

function renderList(models: typeof uiState.data.models): string {
  return `
    <div class="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
      ${models
        .slice(0, 60)
        .map((model) => {
          const caps: string[] = [];
          if (model.supports_reasoning) caps.push("reasoning");
          if (model.supports_tools) caps.push("tools");
          return `
            <div class="flex items-center justify-between rounded-lg border hairline bg-frosted px-3 py-2.5 transition-colors hover:bg-frosted-hover">
              <div class="min-w-0">
                <div class="truncate font-mono text-[13px] text-text">${escapeHtml(model.id)}</div>
                <div class="mt-0.5 text-[11px] text-text-faint">${escapeHtml(model.provider)}</div>
              </div>
              ${caps.length ? `<div class="flex shrink-0 gap-1">${caps.map((c) => `<span class="rounded-full border hairline px-1.5 py-0.5 text-[10px] text-text-muted">${escapeHtml(c)}</span>`).join("")}</div>` : ""}
            </div>
          `;
        })
        .join("")}
    </div>
    ${models.length > 60 ? `<p class="mt-2 text-[12px] text-text-faint">Showing 60 of ${models.length} models. Search to filter.</p>` : ""}
  `;
}

export function bindModels() {
  const input = document.getElementById("model-search") as HTMLInputElement | null;
  input?.addEventListener("input", (e) => {
    uiState.modelSearch = (e.target as HTMLInputElement).value;
    // caller re-renders
  });
}
