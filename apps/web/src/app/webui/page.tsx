import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";

export const metadata: Metadata = {
  title: "Web UI",
};

const steps = [
  "Install Gunmetal globally with npm.",
  "Run `gunmetal setup` once to connect one provider and create a local key.",
  "Run `gunmetal web` to start the local browser surface and open it in your browser.",
  "Use the same machine-local UI to auth providers, sync models, create keys, and inspect request history with token usage.",
];

export default function WebUiPage() {
  return (
    <SiteShell
      eyebrow="Web UI"
      title="Run Gunmetal in the browser without moving state off your machine."
      lede="The hosted site explains Gunmetal. The local Web UI operates it. `gunmetal web` starts the same daemon, then opens the browser surface served from localhost."
    >
      <section className="section-grid">
        <div className="section-head">
          <p className="section-tag">Launch</p>
          <h2>One command opens your local provider console.</h2>
          <p>
            The Web UI is not a hosted dashboard. It is a local surface, served
            by Gunmetal itself, so providers, sessions, keys, request history,
            and token stats stay on the same machine at `127.0.0.1`.
          </p>
          <ol className="step-list">
            {steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>browser flow</span>
          </div>
          <pre>{`npm i -g @dhruv2mars/gunmetal

gunmetal setup
gunmetal web

# local web surface
http://127.0.0.1:4684/app

# local API surface
http://127.0.0.1:4684/v1`}</pre>
        </article>
      </section>

      <section className="section-grid">
        <article className="lane-card">
          <p className="panel-kicker">What it does</p>
          <ul className="stack-list">
            <li>connect providers</li>
            <li>open browser auth for codex and copilot</li>
            <li>save direct API keys for gateway and direct providers</li>
            <li>sync model catalogs</li>
            <li>create, disable, revoke, and delete Gunmetal keys</li>
            <li>inspect request history and token usage</li>
          </ul>
        </article>
        <article className="lane-card">
          <p className="panel-kicker">Design rule</p>
          <p>
            The browser surface is meant to operate Gunmetal, not explain it.
            The public landing page can stay polished and hosted. The Web UI
            stays local, fast, and close to the machine that owns the secrets.
          </p>
        </article>
      </section>
    </SiteShell>
  );
}
