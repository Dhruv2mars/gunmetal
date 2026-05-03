import { escapeHtml } from "../utils";
import { uiState } from "../state";

export function renderToast(): string {
  if (!uiState.toast) return "";
  const { message, tone } = uiState.toast;
  const borderColor = tone === "danger" ? "rgba(255,141,120,0.3)" : tone === "success" ? "rgba(125,211,200,0.3)" : "rgba(226,226,226,0.12)";
  const bgColor = tone === "danger" ? "rgba(255,141,120,0.08)" : tone === "success" ? "rgba(125,211,200,0.08)" : "rgba(255,255,255,0.03)";

  return `
    <div class="fixed bottom-4 right-4 z-[100] max-w-sm animate-fade-up rounded-lg border px-4 py-3 shadow-2xl" style="border-color:${borderColor};background:${bgColor};backdrop-filter:blur(12px)">
      <p class="text-[13px] text-text">${escapeHtml(message)}</p>
    </div>
  `;
}
