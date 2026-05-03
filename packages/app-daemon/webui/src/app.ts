import { loadState } from "./api";
import { uiState, setRenderer, render } from "./state";
import { renderTopbar, bindTopbar } from "./components/topbar";
import { renderSetup } from "./components/setup";
import { renderStats } from "./components/stats";
import { renderActivity, bindActivity } from "./components/activity";
import { renderPlayground, bindPlayground } from "./components/playground";
import { renderProviders, bindProviders } from "./components/providers";
import { renderKeys, bindKeys } from "./components/keys";
import { renderModels, bindModels } from "./components/models";
import { renderToast } from "./components/toast";

function fullRender() {
  const app = document.getElementById("app");
  if (!app) return;

  app.innerHTML = `
    ${renderTopbar()}
    <main class="pb-20">
      ${renderSetup()}
      ${renderStats()}
      ${renderActivity()}
      ${renderPlayground()}
      ${renderProviders()}
      ${renderKeys()}
      ${renderModels()}
    </main>
    ${renderToast()}
  `;

  bindTopbar();
  bindActivity();
  bindPlayground();
  bindProviders();
  bindKeys();
  bindModels();
}

async function refresh() {
  uiState.loading = true;
  try {
    const data = await loadState();
    uiState.data = data;
    if (!uiState.playground.provider && data.profiles.length) {
      uiState.playground.provider = data.profiles[0].provider;
    }
    if (!uiState.playground.model && data.models.length) {
      const models = data.models.filter((m) => m.provider === uiState.playground.provider);
      uiState.playground.model = models[0]?.id || data.models[0].id;
    }
  } catch (err: any) {
    uiState.data = null;
    // We'll show error in UI via empty states
  } finally {
    uiState.loading = false;
    render();
  }
}

function initKeyboard() {
  document.addEventListener("keydown", (e) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    if (e.key === "r" && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      refresh();
    }
    if (e.key === "c" && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      const input = document.getElementById("pg-input") as HTMLTextAreaElement | null;
      input?.focus();
    }
    if (e.key === "Escape") {
      uiState.expandedLogId = null;
      uiState.expandedSection = null;
      render();
    }
  });
}

export function initApp() {
  setRenderer(fullRender);
  window.addEventListener("gm:refresh", () => refresh());
  initKeyboard();
  refresh();
}
