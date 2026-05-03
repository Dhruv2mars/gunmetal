import { escapeHtml } from "../utils";
import { sendPlayground } from "../api";
import { uiState, setToast, render } from "../state";

export function renderPlayground(): string {
  const data = uiState.data;
  const pg = uiState.playground;
  const providers = data
    ? data.profiles.filter((p) => data.models.some((m) => m.provider === p.provider))
    : [];
  const models = data
    ? data.models.filter((m) => m.provider === pg.provider)
    : [];

  const isExpanded = uiState.expandedSection === "playground";

  return `
    <section class="mx-auto max-w-5xl border-y hairline px-4 py-6 md:px-6">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-[18px] font-medium tracking-[-0.02em] text-text">Playground</h2>
          <p class="mt-0.5 text-[13px] text-text-muted">Test models with a Gunmetal key</p>
        </div>
        <button id="toggle-playground" class="tap-target rounded-lg border hairline px-3 py-1.5 text-[13px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]">
          ${isExpanded ? "Collapse" : "Expand"}
        </button>
      </div>

      <div id="playground-body" class="${isExpanded ? "" : "max-h-[200px] overflow-hidden"} transition-all duration-300">
        <div class="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <div>
            <label class="text-label text-text-faint">Gunmetal key</label>
            <input id="pg-key" value="${escapeHtml(pg.key)}" placeholder="gm_..." class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2.5 font-mono text-[13px] text-text placeholder:text-text-faint focus:border-accent" />
          </div>
          <div>
            <label class="text-label text-text-faint">Provider</label>
            <select id="pg-provider" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">
              ${providers.length
                ? providers.map((p) => `<option value="${escapeHtml(p.provider)}"${pg.provider === p.provider ? " selected" : ""}>${escapeHtml(p.name)}</option>`).join("")
                : `<option value="">No providers</option>`}
            </select>
          </div>
          <div>
            <label class="text-label text-text-faint">Model</label>
            <select id="pg-model" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2 font-mono text-[12px] text-text-secondary focus:border-accent">
              ${models.length
                ? models.map((m) => `<option value="${escapeHtml(m.id)}"${pg.model === m.id ? " selected" : ""}>${escapeHtml(m.id)}</option>`).join("")
                : `<option value="">No models</option>`}
            </select>
          </div>
          <div>
            <label class="text-label text-text-faint">Mode</label>
            <select id="pg-mode" class="mt-1.5 h-9 w-full rounded-md border hairline bg-bg px-2 text-[13px] text-text-secondary focus:border-accent">
              <option value="chat"${pg.mode === "chat" ? " selected" : ""}>chat/completions</option>
              <option value="responses"${pg.mode === "responses" ? " selected" : ""}>responses</option>
            </select>
          </div>
        </div>

        <div class="mt-3">
          <div id="pg-messages" class="max-h-[360px] overflow-auto rounded-xl border hairline bg-frosted p-3">
            ${pg.messages.length
              ? pg.messages
                  .map(
                    (m) => `
                      <div class="mb-2 last:mb-0">
                        <div class="mb-1 text-label ${m.role === "user" ? "text-accent" : "text-[#7dbeb3]"}">${m.role}</div>
                        <div class="whitespace-pre-wrap text-[14px] leading-relaxed text-text">${escapeHtml(m.content)}</div>
                      </div>
                    `
                  )
                  .join("")
              : `<div class="flex h-24 items-center justify-center text-[13px] text-text-muted">Choose a model and send a message</div>`}
          </div>
          <form id="pg-form" class="mt-3 flex gap-2">
            <textarea id="pg-input" placeholder="Ask something..." rows="2" class="min-h-[48px] flex-1 resize-y rounded-lg border hairline bg-bg px-3 py-2 text-[14px] text-text placeholder:text-text-faint focus:border-accent" ${pg.running ? "disabled" : ""}></textarea>
            <button type="submit" class="tap-target self-end rounded-lg border border-accent/40 bg-accent-soft px-4 py-2 text-[13px] font-medium text-accent transition-all hover:bg-accent/20 active:scale-[0.97]" ${pg.running || !pg.model ? "disabled" : ""}>
              ${pg.running ? "..." : "Send"}
            </button>
            <button type="button" id="pg-clear" class="tap-target self-end rounded-lg border hairline px-3 py-2 text-[13px] text-text-secondary transition-all hover:bg-frosted-hover active:scale-[0.97]" ${pg.running || pg.messages.length === 0 ? "disabled" : ""}>Clear</button>
          </form>
        </div>
      </div>
    </section>
  `;
}

export function bindPlayground() {
  const keyInput = document.getElementById("pg-key") as HTMLInputElement | null;
  const providerSelect = document.getElementById("pg-provider") as HTMLSelectElement | null;
  const modelSelect = document.getElementById("pg-model") as HTMLSelectElement | null;
  const modeSelect = document.getElementById("pg-mode") as HTMLSelectElement | null;

  keyInput?.addEventListener("input", (e) => {
    uiState.playground.key = (e.target as HTMLInputElement).value;
  });

  providerSelect?.addEventListener("change", (e) => {
    uiState.playground.provider = (e.target as HTMLSelectElement).value;
    const data = uiState.data;
    if (data) {
      const models = data.models.filter((m) => m.provider === uiState.playground.provider);
      uiState.playground.model = models[0]?.id || "";
    }
    render();
  });

  modelSelect?.addEventListener("change", (e) => {
    uiState.playground.model = (e.target as HTMLSelectElement).value;
  });

  modeSelect?.addEventListener("change", (e) => {
    uiState.playground.mode = (e.target as HTMLSelectElement).value as "chat" | "responses";
  });

  document.getElementById("toggle-playground")?.addEventListener("click", () => {
    uiState.expandedSection = uiState.expandedSection === "playground" ? null : "playground";
    render();
  });

  document.getElementById("pg-clear")?.addEventListener("click", () => {
    uiState.playground.messages = [];
    uiState.playground.usage = null;
    uiState.playground.lastDurationMs = null;
    render();
  });

  document.getElementById("pg-form")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const input = document.getElementById("pg-input") as HTMLTextAreaElement | null;
    const content = input?.value.trim() || "";
    const pg = uiState.playground;

    if (!pg.key) { setToast("Paste a Gunmetal key first", "danger"); return; }
    if (!pg.model) { setToast("Sync a model first", "danger"); return; }
    if (!content) { setToast("Enter a message", "danger"); return; }

    if (pg.historyMode === "single") pg.messages = [];
    pg.messages.push({ role: "user", content });
    pg.running = true;
    pg.usage = null;
    if (input) input.value = "";
    render();

    try {
      const response = await sendPlayground(pg.mode, pg.key, pg.model, pg.messages);
      if (!response.ok) {
        const text = await response.text();
        let msg = "request failed";
        try { msg = JSON.parse(text)?.error?.message || msg; } catch {}
        throw new Error(msg);
      }

      pg.messages.push({ role: "assistant", content: "" });
      render();

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();
      if (!reader) throw new Error("No response stream");

      let buffer = "";
      let assistantText = "";
      const startedAt = performance.now();

      while (true) {
        const { value, done } = await reader.read();
        buffer += decoder.decode(value || new Uint8Array(), { stream: !done });

        let boundary = buffer.indexOf("\n\n");
        while (boundary !== -1) {
          const chunk = buffer.slice(0, boundary);
          buffer = buffer.slice(boundary + 2);
          const lines = chunk.split(/\r?\n/);
          let event = "message";
          const dataLines: string[] = [];

          for (const line of lines) {
            if (line.startsWith("event:")) event = line.slice(6).trim();
            else if (line.startsWith("data:")) dataLines.push(line.slice(5).trimStart());
          }

          if (dataLines.length && dataLines.join("") !== "[DONE]") {
            const parsed = JSON.parse(dataLines.join("\n"));
            if (pg.mode === "chat") {
              const delta = parsed?.choices?.[0]?.delta?.content || "";
              if (delta) { assistantText += delta; }
            } else if (event === "response.output_text.delta") {
              assistantText += parsed.delta || "";
            } else if (event === "response.completed") {
              assistantText = parsed?.response?.output_text || assistantText;
              pg.usage = parsed?.response?.usage || null;
            }
            const last = pg.messages[pg.messages.length - 1];
            if (last && last.role === "assistant") last.content = assistantText;
            render();
          }
          boundary = buffer.indexOf("\n\n");
        }
        if (done) break;
      }

      pg.lastDurationMs = Math.round(performance.now() - startedAt);
      setToast("Response received", "success");
    } catch (err: any) {
      pg.messages = pg.messages.slice(0, Math.max(0, pg.messages.length - 2));
      setToast(err.message || "Request failed", "danger");
    } finally {
      pg.running = false;
      render();
    }
  });
}
