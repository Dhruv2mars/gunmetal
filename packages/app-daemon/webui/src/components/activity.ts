import { escapeHtml, formatRelativeTime, formatDuration } from "../utils";
import { uiState, logMatchesFilters } from "../state";

export function renderActivity(): string {
  const data = uiState.data;
  if (!data) return "";

  const filtered = data.logs.filter(logMatchesFilters);
  const visible =
    uiState.requestFilters.limit === "all"
      ? filtered
      : filtered.slice(0, Number(uiState.requestFilters.limit) || 8);

  const providerOptions = [
    `<option value="all">All providers</option>`,
    ...data.provider_summaries.map((item) => {
      const value = `${item.provider}::${item.profile_name || ""}`;
      return `<option value="${escapeHtml(value)}"${uiState.requestFilters.provider === value ? " selected" : ""}>${escapeHtml(item.label)}</option>`;
    }),
  ].join("");

  return `
    <section class="mx-auto max-w-5xl px-4 py-6 md:px-6">
      <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 class="text-[18px] font-medium tracking-[-0.02em] text-text">Activity</h2>
          <p class="mt-0.5 text-[13px] text-text-muted">Recent requests and traffic</p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <select id="filter-provider" class="h-8 rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">${providerOptions}</select>
          <select id="filter-status" class="h-8 rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">
            <option value="all"${uiState.requestFilters.status === "all" ? " selected" : ""}>All statuses</option>
            <option value="success"${uiState.requestFilters.status === "success" ? " selected" : ""}>Success</option>
            <option value="error"${uiState.requestFilters.status === "error" ? " selected" : ""}>Errors</option>
          </select>
          <input id="filter-query" value="${escapeHtml(uiState.requestFilters.query)}" placeholder="Search..." class="h-8 w-40 rounded-md border hairline bg-bg px-2 text-[13px] text-text placeholder:text-text-faint focus:border-accent" />
          <select id="filter-limit" class="h-8 rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">
            <option value="8"${uiState.requestFilters.limit === "8" ? " selected" : ""}>Latest 8</option>
            <option value="24"${uiState.requestFilters.limit === "24" ? " selected" : ""}>Latest 24</option>
            <option value="all"${uiState.requestFilters.limit === "all" ? " selected" : ""}>All</option>
          </select>
        </div>
      </div>

      ${visible.length === 0 ? renderEmpty() : renderList(visible)}
    </section>
  `;
}

function renderEmpty(): string {
  const data = uiState.data;
  const hasLogs = data && data.logs.length > 0;
  return `
    <div class="mt-6 flex flex-col items-center justify-center rounded-xl border border-dashed hairline py-16">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-text-faint"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
      <p class="mt-3 text-[14px] text-text-muted">${hasLogs ? "No requests match the current filters." : "No traffic yet."}</p>
      ${!hasLogs ? `<p class="mt-1 text-[13px] text-text-faint">Send a request from the playground or point an app at the local API.</p>` : ""}
    </div>
  `;
}

function renderList(logs: typeof uiState.data.logs): string {
  return `
    <div class="mt-4 divide-y hairline">
      ${logs
        .map((log) => {
          const isError = (log.status_code ?? 0) >= 400 || log.error_message;
          const isExpanded = uiState.expandedLogId === log.id;

          return `
            <div class="group cursor-pointer transition-colors duration-150 hover:bg-frosted" data-log-id="${escapeHtml(log.id)}">
              <div class="flex items-center gap-3 py-3 px-2">
                <span class="h-2 w-2 shrink-0 rounded-full ${isError ? "bg-[#ff8d78]" : "bg-[#7dbeb3]"}"></span>
                <div class="min-w-0 flex-1">
                  <div class="flex items-baseline gap-2">
                    <span class="truncate text-[14px] text-text">${escapeHtml(log.model)}</span>
                    <span class="shrink-0 text-[12px] text-text-muted">${escapeHtml(log.profile_name || log.provider)}</span>
                  </div>
                  <div class="mt-0.5 flex items-center gap-2 text-[12px] text-text-faint">
                    <span>${escapeHtml(log.request_mode || log.endpoint)}</span>
                    <span>·</span>
                    <span>${formatDuration(log.duration_ms)}</span>
                    ${log.total_tokens ? `<span>·</span><span>${log.total_tokens} tokens</span>` : ""}
                  </div>
                </div>
                <div class="shrink-0 text-right">
                  <div class="text-[12px] text-text-muted">${formatRelativeTime(log.started_at)}</div>
                  <div class="mt-0.5 font-mono text-[12px] ${isError ? "text-[#ff8d78]" : "text-text-faint"}">${log.status_code ?? "—"}</div>
                </div>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="shrink-0 text-text-faint transition-transform duration-200 ${isExpanded ? "rotate-180" : ""}"><polyline points="6 9 12 15 18 9"/></svg>
              </div>
              ${isExpanded ? renderLogDetail(log) : ""}
            </div>
          `;
        })
        .join("")}
    </div>
    <p class="mt-2 text-[12px] text-text-faint">Showing ${logs.length} of ${uiState.data.logs.filter(logMatchesFilters).length} matching requests</p>
  `;
}

function renderLogDetail(log: typeof uiState.data.logs[0]): string {
  return `
    <div class="border-t hairline bg-frosted px-2 pb-4 pt-3 animate-fade-in">
      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <div><span class="text-label text-text-faint">Status</span><div class="mt-1 text-[13px] text-text">${escapeHtml(String(log.status_code ?? "pending"))}${log.error_message ? ` <span class="text-[#ff8d78]">· ${escapeHtml(log.error_message)}</span>` : ""}</div></div>
        <div><span class="text-label text-text-faint">Latency</span><div class="mt-1 text-[13px] text-text">${formatDuration(log.duration_ms)}</div></div>
        <div><span class="text-label text-text-faint">Tokens</span><div class="mt-1 text-[13px] text-text">in ${log.input_tokens ?? 0} · out ${log.output_tokens ?? 0} · total ${log.total_tokens ?? 0}</div></div>
        <div><span class="text-label text-text-faint">Key</span><div class="mt-1 text-[13px] text-text">${escapeHtml(log.key_name || "unknown")}</div></div>
        <div><span class="text-label text-text-faint">Endpoint</span><div class="mt-1 font-mono text-[12px] text-text-secondary">${escapeHtml(log.endpoint)}</div></div>
        <div><span class="text-label text-text-faint">Time</span><div class="mt-1 text-[13px] text-text">${escapeHtml(new Date(log.started_at).toLocaleString())}</div></div>
      </div>
    </div>
  `;
}

export function bindActivity() {
  document.querySelectorAll("[data-log-id]").forEach((el) => {
    el.addEventListener("click", () => {
      const id = el.getAttribute("data-log-id");
      uiState.expandedLogId = uiState.expandedLogId === id ? null : id;
      // re-render handled by caller
    });
  });

  const updateFilter = (key: string, value: string) => {
    (uiState.requestFilters as Record<string, string>)[key] = value;
    // caller re-renders
  };

  document.getElementById("filter-provider")?.addEventListener("change", (e) => updateFilter("provider", (e.target as HTMLSelectElement).value));
  document.getElementById("filter-status")?.addEventListener("change", (e) => updateFilter("status", (e.target as HTMLSelectElement).value));
  document.getElementById("filter-query")?.addEventListener("input", (e) => updateFilter("query", (e.target as HTMLInputElement).value));
  document.getElementById("filter-limit")?.addEventListener("change", (e) => updateFilter("limit", (e.target as HTMLSelectElement).value));
}
