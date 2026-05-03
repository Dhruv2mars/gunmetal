import { escapeHtml, formatNumber } from "../utils";
import { uiState } from "../state";

export function renderStats(): string {
  const data = uiState.data;
  if (!data) return "";

  const items = [
    { label: "Providers", value: data.counts.profiles },
    { label: "Models", value: data.counts.models },
    { label: "Keys", value: data.counts.keys },
    { label: "Requests", value: data.counts.logs },
  ];

  return `
    <section class="mx-auto max-w-5xl border-b hairline px-4 py-4 md:px-6">
      <div class="flex flex-wrap items-baseline gap-x-6 gap-y-2">
        ${items
          .map(
            (item) => `
              <div class="flex items-baseline gap-2">
                <span class="font-mono text-[22px] font-medium leading-none text-text">${formatNumber(item.value)}</span>
                <span class="text-label text-text-muted">${escapeHtml(item.label)}</span>
              </div>
            `
          )
          .join("")}
      </div>
    </section>
  `;
}
