"use client";

import { useMemo, useState } from "react";

import {
  perfPresets,
  roundMs,
  summarizeRuns,
  type PerfPreset,
  type PerfTransport,
} from "./shared";

type RunRecord = {
  id: string;
  scenario: string;
  label: string;
  provider: string;
  route: string;
  transport: PerfTransport;
  ok: boolean;
  status: number;
  uiElapsedMs: number;
  serverElapsedMs: number;
  content: string;
  error?: string;
};

type FormState = {
  presetId: string;
  transport: PerfTransport;
  apiKey: string;
  baseUrl: string;
  model: string;
  prompt: string;
  temperature: string;
  maxTokens: string;
};

const presetOptions = perfPresets.map((preset) => ({
  value: preset.model,
  label: `${preset.provider} · ${preset.model}`,
}));

function applyPreset(preset: PerfPreset, apiKey: string): FormState {
  return {
    presetId: preset.id,
    transport: preset.transport,
    apiKey,
    baseUrl: preset.baseUrl,
    model: preset.model,
    prompt: preset.prompt,
    temperature: String(preset.temperature),
    maxTokens: String(preset.maxTokens),
  };
}

export function PerfLabClient() {
  const initialPreset = perfPresets[0];
  const [form, setForm] = useState<FormState>(() =>
    applyPreset(initialPreset, ""),
  );
  const [runs, setRuns] = useState<RunRecord[]>([]);
  const [pending, setPending] = useState(false);
  const [responseText, setResponseText] = useState("");
  const [errorText, setErrorText] = useState("");

  const currentPreset =
    perfPresets.find((preset) => preset.id === form.presetId) ?? initialPreset;
  const scenarioKey = `${form.transport}|${form.baseUrl}|${form.model}|${form.prompt}|${form.temperature}|${form.maxTokens}`;

  const currentRuns = useMemo(
    () => runs.filter((run) => run.scenario === scenarioKey && run.ok),
    [runs, scenarioKey],
  );
  const currentSummary = useMemo(
    () => summarizeRuns(currentRuns.map((run) => run.uiElapsedMs)),
    [currentRuns],
  );

  async function submit() {
    setPending(true);
    setErrorText("");

    const started = performance.now();
    try {
      const response = await fetch("/api/perf-chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          transport: form.transport,
          apiKey: form.apiKey,
          baseUrl: form.baseUrl,
          model: form.model,
          prompt: form.prompt,
          temperature: Number(form.temperature),
          maxTokens: Number(form.maxTokens),
        }),
      });
      const payload = (await response.json()) as {
        ok: boolean;
        status: number;
        content: string;
        error?: string;
        serverElapsedMs: number;
      };
      const uiElapsedMs = roundMs(performance.now() - started);
      const record: RunRecord = {
        id: crypto.randomUUID(),
        scenario: scenarioKey,
        label: currentPreset.label,
        provider: currentPreset.provider,
        route: currentPreset.route,
        transport: form.transport,
        ok: payload.ok,
        status: payload.status,
        uiElapsedMs,
        serverElapsedMs: payload.serverElapsedMs,
        content: payload.content,
        error: payload.error,
      };

      setRuns((previous) => [record, ...previous]);
      setResponseText(payload.content);
      setErrorText(payload.ok ? "" : payload.error ?? "Request failed.");
    } catch (error) {
      const uiElapsedMs = roundMs(performance.now() - started);
      const message =
        error instanceof Error ? error.message : "Request failed.";
      setErrorText(message);
      setRuns((previous) => [
        {
          id: crypto.randomUUID(),
          scenario: scenarioKey,
          label: currentPreset.label,
          provider: currentPreset.provider,
          route: currentPreset.route,
          transport: form.transport,
          ok: false,
          status: 0,
          uiElapsedMs,
          serverElapsedMs: 0,
          content: "",
          error: message,
        },
        ...previous,
      ]);
    } finally {
      setPending(false);
    }
  }

  function updateField<Key extends keyof FormState>(key: Key, value: FormState[Key]) {
    setForm((previous) => ({ ...previous, [key]: value }));
  }

  function onPromptKeyDown(event: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key !== "Enter" || event.shiftKey || pending) {
      return;
    }

    event.preventDefault();
    void submit();
  }

  return (
    <section className="perf-lab-grid">
      <article className="lane-card perf-panel">
        <div className="section-head">
          <p className="section-tag">Scenarios</p>
          <h2>Preset the exact provider and route before you type.</h2>
          <p>
            Every preset fixes the target model and default config so the only
            thing that changes is the path: direct provider vs Gunmetal.
          </p>
        </div>
        <div className="perf-preset-grid">
          {perfPresets.map((preset) => (
            <button
              key={preset.id}
              className={`perf-preset ${form.presetId === preset.id ? "perf-preset-active" : ""}`}
              onClick={() => setForm((previous) => applyPreset(preset, previous.apiKey))}
              type="button"
            >
              <span>{preset.label}</span>
              <small>{preset.note}</small>
            </button>
          ))}
        </div>
      </article>

      <article className="code-panel perf-panel">
        <div className="code-head">
          <span>Local-only harness</span>
          <span>{currentPreset.provider}</span>
        </div>
        <p className="perf-note">
          This page only works on localhost. The route rejects hosted traffic so
          secrets never leave the same machine that runs the benchmark.
        </p>

        <div className="perf-field-grid">
          <label className="perf-field">
            <span>Transport</span>
            <select
              aria-label="Transport"
              value={form.transport}
              onChange={(event) =>
                updateField("transport", event.target.value as PerfTransport)
              }
            >
              <option value="openai-compatible">openai-compatible</option>
              <option value="codex-direct">codex-direct</option>
            </select>
          </label>

          <label className="perf-field">
            <span>Model</span>
            <input
              aria-label="Model"
              list="perf-models"
              value={form.model}
              onChange={(event) => updateField("model", event.target.value)}
            />
            <datalist id="perf-models">
              {presetOptions.map((option) => (
                <option key={option.label} value={option.value}>
                  {option.label}
                </option>
              ))}
            </datalist>
          </label>

          <label className="perf-field perf-field-wide">
            <span>API Key</span>
            <input
              aria-label="API Key"
              disabled={form.transport === "codex-direct"}
              placeholder={
                form.transport === "codex-direct"
                  ? "Codex direct uses your local Codex login."
                  : "Paste upstream or Gunmetal key."
              }
              type="password"
              value={form.apiKey}
              onChange={(event) => updateField("apiKey", event.target.value)}
            />
          </label>

          <label className="perf-field perf-field-wide">
            <span>Base URL</span>
            <input
              aria-label="Base URL"
              disabled={form.transport === "codex-direct"}
              placeholder="https://provider.example/v1"
              value={form.baseUrl}
              onChange={(event) => updateField("baseUrl", event.target.value)}
            />
          </label>

          <label className="perf-field">
            <span>Temperature</span>
            <input
              aria-label="Temperature"
              inputMode="decimal"
              value={form.temperature}
              onChange={(event) => updateField("temperature", event.target.value)}
            />
          </label>

          <label className="perf-field">
            <span>Max Tokens</span>
            <input
              aria-label="Max Tokens"
              inputMode="numeric"
              value={form.maxTokens}
              onChange={(event) => updateField("maxTokens", event.target.value)}
            />
          </label>
        </div>

        <label className="perf-field perf-field-wide">
          <span>Prompt</span>
          <textarea
            aria-label="Prompt"
            className="perf-textarea"
            value={form.prompt}
            onChange={(event) => updateField("prompt", event.target.value)}
            onKeyDown={onPromptKeyDown}
          />
        </label>

        <div className="button-row">
          <button
            className="button button-primary"
            disabled={pending}
            onClick={() => void submit()}
            type="button"
          >
            {pending ? "Running…" : "Send Prompt"}
          </button>
          <button
            className="button button-secondary"
            onClick={() => {
              setRuns([]);
              setResponseText("");
              setErrorText("");
            }}
            type="button"
          >
            Clear Runs
          </button>
        </div>
      </article>

      <article className="lane-card perf-panel">
        <div className="code-head">
          <span>Current scenario</span>
          <span>{currentPreset.route}</span>
        </div>
        <div className="chip-row">
          <span className="chip chip-wide">
            provider: {currentPreset.provider}
          </span>
          <span className="chip chip-wide">
            route: {currentPreset.route}
          </span>
          <span className="chip chip-wide">
            model: {form.model}
          </span>
        </div>
        {currentSummary ? (
          <div className="perf-summary-grid">
            <div className="signal-panel">
              <p className="panel-kicker">UI min</p>
              <p className="stat-line">{currentSummary.minMs} ms</p>
            </div>
            <div className="signal-panel">
              <p className="panel-kicker">UI median</p>
              <p className="stat-line">{currentSummary.medianMs} ms</p>
            </div>
            <div className="signal-panel">
              <p className="panel-kicker">UI max</p>
              <p className="stat-line">{currentSummary.maxMs} ms</p>
            </div>
          </div>
        ) : (
          <p className="perf-note">
            No successful runs yet. Focus the prompt box and press Enter to add
            the first one.
          </p>
        )}

        <div className="perf-output-grid">
          <div className="perf-response">
            <p className="panel-kicker">Last response</p>
            <pre>{responseText || "No response yet."}</pre>
          </div>
          <div className="perf-response">
            <p className="panel-kicker">Last error</p>
            <pre>{errorText || "No error."}</pre>
          </div>
        </div>
      </article>

      <article className="code-panel perf-panel perf-panel-wide">
        <div className="code-head">
          <span>Run history</span>
          <span>{runs.length} total</span>
        </div>
        <div className="perf-table-wrap">
          <table className="perf-table">
            <thead>
              <tr>
                <th>Scenario</th>
                <th>Status</th>
                <th>UI ms</th>
                <th>Server ms</th>
                <th>Output</th>
              </tr>
            </thead>
            <tbody>
              {runs.length === 0 ? (
                <tr>
                  <td colSpan={5}>No runs yet.</td>
                </tr>
              ) : (
                runs.map((run) => (
                  <tr key={run.id}>
                    <td>{run.label}</td>
                    <td>{run.status}</td>
                    <td>{run.uiElapsedMs}</td>
                    <td>{run.serverElapsedMs}</td>
                    <td>{run.ok ? run.content : run.error ?? "failed"}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </article>
    </section>
  );
}
