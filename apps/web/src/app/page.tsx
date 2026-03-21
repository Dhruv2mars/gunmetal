import Link from "next/link";

import { SiteShell } from "@/components/site-shell";
import {
  apiSnippet,
  installSnippet,
  moduleCards,
  providerLanes,
} from "@/lib/site-content";

export default function Home() {
  return (
    <SiteShell
      eyebrow="New repo. New product. Local-first by design."
      title="The dashboard and local API for every AI provider you already use."
      lede="Gunmetal connects subscription providers, gateways, and direct API keys behind one local endpoint, one local key system, one CLI, and one TUI."
    >
      <section className="hero-grid">
        <article className="signal-panel signal-panel-large">
          <div className="signal-rail">
            <span>LOCAL</span>
            <span>EXPLICIT</span>
            <span>MODULAR</span>
          </div>
          <div className="signal-copy">
            <p className="stat-line">One download. One binary. One local service.</p>
            <p>
              Gunmetal does not resell subscriptions. It gives your machine a
              clean local inference surface and routes requests to the upstream
              account you connected on purpose.
            </p>
            <div className="button-row">
              <Link className="button button-primary" href="/install">
                Install Gunmetal
              </Link>
              <Link className="button button-secondary" href="/docs">
                Read the contract
              </Link>
            </div>
          </div>
        </article>

        <article className="signal-panel">
          <p className="panel-kicker">Provider lanes</p>
          <ul className="stack-list">
            <li>subscription: codex, copilot</li>
            <li>gateway: openrouter, zen</li>
            <li>direct key: openai, azure, nvidia</li>
          </ul>
          <p className="panel-note">
            Routing is explicit only. No policy mode. No hidden alias layer.
          </p>
        </article>
      </section>

      <section className="section-grid">
        <div className="section-head">
          <p className="section-tag">Architecture</p>
          <h2>Built as modules, shipped as one product.</h2>
        </div>
        <div className="card-grid">
          {moduleCards.map((card) => (
            <article key={card.title} className="card">
              <h3>{card.title}</h3>
              <p>{card.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-grid">
        <div className="section-head">
          <p className="section-tag">Provider model</p>
          <h2>Three provider classes. One local contract.</h2>
        </div>
        <div className="lane-grid">
          {providerLanes.map((lane) => (
            <article key={lane.title} className="lane-card">
              <p className="panel-kicker">{lane.title}</p>
              <p>{lane.body}</p>
              <div className="chip-row">
                {lane.items.map((item) => (
                  <span key={item} className="chip">
                    {item}
                  </span>
                ))}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">First run</p>
          <h2>Install it. Launch it. Point apps at it.</h2>
          <p>
            The fastest path is `gunmetal setup`, then `gunmetal start`, then
            one `/v1/models` request with the key setup created. After that,
            point apps at Gunmetal instead of the upstream provider.
          </p>
        </div>
        <div className="code-stack">
          <article className="code-panel">
            <div className="code-head">
              <span>install + local control</span>
            </div>
            <pre>{installSnippet}</pre>
          </article>
          <article className="code-panel">
            <div className="code-head">
              <span>OpenAI-style compatibility</span>
            </div>
            <pre>{apiSnippet}</pre>
          </article>
        </div>
      </section>

      <section className="warning-band">
        <p className="section-tag">Compatibility rule</p>
        <h2>Gunmetal works when the app talks to Gunmetal.</h2>
        <p>
          The app needs a custom base URL, a custom API key field, and support
          for arbitrary model names. If the app hardcodes an upstream endpoint,
          Gunmetal cannot intercept it.
        </p>
        <div className="button-row">
          <Link className="button button-secondary" href="/docs">
            See full API syntax
          </Link>
          <Link className="button button-secondary" href="/changelog">
            View current build
          </Link>
        </div>
      </section>
    </SiteShell>
  );
}
