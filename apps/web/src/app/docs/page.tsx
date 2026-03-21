import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";
import {
  apiSnippet,
  docsSections,
  openAiCompatSnippet,
  responsesSnippet,
} from "@/lib/site-content";

export const metadata: Metadata = {
  title: "Docs",
};

const endpoints = [
  "GET /health",
  "GET /v1/models",
  "POST /v1/chat/completions",
  "POST /v1/responses",
];

export default function DocsPage() {
  return (
    <SiteShell
      eyebrow="Docs"
      title="The public contract is small on purpose."
      lede="Gunmetal stays useful by looking standard from the outside while keeping providers explicit on the inside."
    >
      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">Start here</p>
          <h2>First prove /v1/models and /v1/chat/completions.</h2>
          <p>
            A normal user path is short: `gunmetal setup`, `gunmetal start`,
            then one models call and one chat call. That proves the key, base
            URL, and provider/model syntax all line up.
          </p>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>OpenAI-compatible app</span>
          </div>
          <pre>{openAiCompatSnippet}</pre>
        </article>
      </section>

      <section className="section-grid">
        <div className="card-grid">
          {docsSections.map((section) => (
            <article key={section.title} className="card">
              <h2>{section.title}</h2>
              <p>{section.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">Endpoints</p>
          <h2>Support both chat/completions and responses.</h2>
          <ul className="stack-list">
            {endpoints.map((endpoint) => (
              <li key={endpoint}>{endpoint}</li>
            ))}
          </ul>
          <p>
            Chat completions lands first for maximum compatibility. Responses is
            there for apps that want the newer surface.
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
          <p className="section-tag">Responses</p>
          <h2>Same local key. Same local host. Different request shape.</h2>
          <p>
            Use the responses endpoint when your client expects that newer API
            style. The routing rule does not change: always send the request to
            Gunmetal, not the upstream provider.
          </p>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>responses</span>
          </div>
          <pre>{responsesSnippet}</pre>
        </article>
      </section>
    </SiteShell>
  );
}
