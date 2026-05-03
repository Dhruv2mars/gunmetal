import { escapeHtml } from "../utils";
import { uiState } from "../state";

export function renderSetup(): string {
  const data = uiState.data;
  if (!data) return "";

  const steps = [
    { label: "Connect provider", done: data.setup.provider_ready, count: data.counts.profiles },
    { label: "Sync models", done: data.setup.models_ready, count: data.counts.models },
    { label: "Create key", done: data.setup.key_ready, count: data.counts.keys },
    { label: "Send request", done: data.setup.traffic_ready, count: data.counts.logs },
  ];

  const allDone = steps.every((s) => s.done);

  if (allDone) {
    return `
      <section class="mx-auto max-w-5xl px-4 py-3 md:px-6">
        <div class="flex items-center gap-3 rounded-lg border border-success-border bg-success-bg px-4 py-2.5">
          <span class="h-1.5 w-1.5 rounded-full" style="background:#7dbeb3"></span>
          <span class="text-[13px] text-text-secondary">All set. ${escapeHtml(data.setup.next_step)}</span>
        </div>
      </section>
    `;
  }

  return `
    <section class="mx-auto max-w-5xl px-4 py-4 md:px-6">
      <div class="flex flex-wrap items-center gap-2">
        ${steps
          .map((step, i) => {
            const isLast = i === steps.length - 1;
            return `
              <div class="flex items-center gap-2">
                <div class="flex items-center gap-2 rounded-lg border px-3 py-2 ${
                  step.done
                    ? "border-success-border bg-success-bg"
                    : "hairline-strong bg-frosted"
                }">
                  <span class="flex h-5 w-5 items-center justify-center rounded-full text-[11px] font-medium ${
                    step.done ? "bg-[#7dbeb3] text-bg" : "border hairline text-text-muted"
                  }">${step.done ? "✓" : i + 1}</span>
                  <span class="text-[13px] ${step.done ? "text-text-secondary" : "text-text"}">${escapeHtml(step.label)}</span>
                  ${step.done ? `<span class="font-mono text-[12px] text-text-muted">${step.count}</span>` : ""}
                </div>
                ${!isLast ? `<span class="text-text-faint">→</span>` : ""}
              </div>
            `;
          })
          .join("")}
      </div>
      <p class="mt-2 text-[13px] text-text-muted">${escapeHtml(data.setup.next_step)}</p>
    </section>
  `;
}
