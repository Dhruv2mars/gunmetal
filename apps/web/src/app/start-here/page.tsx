import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";
import {
  apiSnippet,
  installSnippet,
  openAiCompatSnippet,
  responsesSnippet,
} from "@/lib/site-content";

export const metadata: Metadata = {
  title: "Start Here",
};

const steps = [
  "Install Gunmetal with npm.",
  "Run `gunmetal setup` and finish one provider flow.",
  "Run `gunmetal start`.",
  "Call `GET /v1/models` with your new Gunmetal key.",
  "Call `POST /v1/chat/completions` with one provider/model id.",
];

export default function StartHerePage() {
  return (
    <SiteShell
      eyebrow="Start here"
      title="Five moves from install to first real request."
      lede="This is the shortest real path: install Gunmetal, connect one provider, get one key, prove it works, then point apps at the same local base URL."
    >
      <section className="section-grid">
        <div className="section-head">
          <p className="section-tag">Golden path</p>
          <h2>Do these in order.</h2>
          <ol className="step-list">
            {steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </div>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">Install + setup</p>
          <h2>One install. One setup flow.</h2>
          <p>
            `gunmetal setup` is the default path. It connects one provider,
            checks auth, syncs models, creates one key, and ends with a
            working test command.
          </p>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>shell</span>
          </div>
          <pre>{installSnippet}</pre>
        </article>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">First request</p>
          <h2>Confirm /v1/models, then /v1/chat/completions.</h2>
          <p>
            Gunmetal keys are local Gunmetal keys. They work only because the
            request goes to Gunmetal at `http://127.0.0.1:4684/v1`.
          </p>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>chat/completions</span>
          </div>
          <pre>{apiSnippet}</pre>
        </article>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">Compatibility</p>
          <h2>OpenAI-style apps work when they can target Gunmetal.</h2>
          <p>
            The app needs a custom base URL field, a custom API key field, and
            arbitrary model names. If it hardcodes the upstream endpoint,
            Gunmetal cannot intercept that traffic.
          </p>
        </div>
        <div className="code-stack">
          <article className="code-panel">
            <div className="code-head">
              <span>OpenAI-compatible app</span>
            </div>
            <pre>{openAiCompatSnippet}</pre>
          </article>
          <article className="code-panel">
            <div className="code-head">
              <span>responses</span>
            </div>
            <pre>{responsesSnippet}</pre>
          </article>
        </div>
      </section>
    </SiteShell>
  );
}
