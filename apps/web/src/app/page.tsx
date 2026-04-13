import Link from "next/link";

import { installSnippet, providerLanes } from "@/lib/site-content";

const proofPoints = [
  "Bring your own providers",
  "One local API endpoint",
  "Browser UI, CLI, and TUI included",
  "Request history and token stats stay local",
];

const operatingSteps = [
  {
    title: "Connect a provider",
    body: "Finish browser auth for subscriptions or save one upstream API key for gateway and direct providers.",
  },
  {
    title: "Sync models and mint a key",
    body: "Gunmetal keeps a local model registry and gives you one Gunmetal key for the apps that will call it.",
  },
  {
    title: "Point apps at Gunmetal",
    body: "Use a custom base URL, your Gunmetal key, and an explicit provider/model id like codex/gpt-5.4.",
  },
];

const surfacePanels = [
  {
    title: "Hosted front door",
    body: "The site explains the install path, the compatibility rule, and the exact product story without pretending there is a hosted relay.",
  },
  {
    title: "Local operator surfaces",
    body: "Use the browser UI when you want guided setup, the CLI when you want exact commands, and the TUI when you want the same flow from a terminal.",
  },
  {
    title: "Real local gateway",
    body: "Gunmetal runs on your machine, issues Gunmetal keys, serves /v1 models and inference routes, and keeps request history next to the machine that owns the access.",
  },
];

const capabilityBands = [
  "Supports both chat/completions and responses",
  "Playground for real key and model testing",
  "Same state across browser, TUI, and CLI",
  "Explicit provider/model ids only",
];

export default function HomePage() {
  return (
    <div className="home-page">
      <header className="site-frame site-header site-header-home">
        <Link className="brand-mark" href="/">
          <span className="brand-chip">GM</span>
          <span className="brand-copy">
            <strong>Gunmetal</strong>
            <span>Local inference middle layer</span>
          </span>
        </Link>
        <nav className="site-nav" aria-label="Primary">
          <Link href="/install">Install</Link>
          <Link href="/start-here">Start Here</Link>
          <Link href="/webui">Web UI</Link>
          <Link href="/docs">Docs</Link>
          <a
            className="nav-cta"
            href="https://github.com/Dhruv2mars/gunmetal"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
        </nav>
      </header>

      <main>
        <section className="poster-hero">
          <div className="poster-copy">
            <p className="eyebrow">Gunmetal super app</p>
            <h1>Turn the AI access you already pay for into one local API.</h1>
            <p className="lede">
              Gunmetal sits between your apps and your providers. Connect one
              provider, mint one Gunmetal key, keep one local endpoint, and use
              the browser UI, CLI, or TUI to inspect what actually happened.
            </p>
            <div className="button-row">
              <Link className="button button-primary" href="/install">
                Install Gunmetal
              </Link>
              <Link className="button button-secondary" href="/start-here">
                Follow the golden path
              </Link>
            </div>
            <div className="proof-strip home-proof-strip">
              {proofPoints.map((item) => (
                <span key={item}>{item}</span>
              ))}
            </div>
          </div>

          <div className="poster-stage">
            <div className="stage-window">
              <div className="stage-topline">
                <span>Install once</span>
                <span>operate locally</span>
              </div>
              <div className="stage-columns">
                <div>
                  <p className="panel-kicker">What public users actually do</p>
                  <ol className="step-list home-step-list">
                    {operatingSteps.map((step) => (
                      <li key={step.title}>
                        <strong>{step.title}</strong>
                        <span>{step.body}</span>
                      </li>
                    ))}
                  </ol>
                </div>
                <div className="stage-command mono">
                  <span>npm i -g @dhruv2mars/gunmetal</span>
                  <span>gunmetal setup</span>
                  <span>gunmetal web</span>
                  <span>gunmetal chat</span>
                  <span>OPENAI_BASE_URL=http://127.0.0.1:4684/v1</span>
                </div>
              </div>
            </div>
            <div className="proof-strip">
              {capabilityBands.map((item) => (
                <span key={item}>{item}</span>
              ))}
            </div>
          </div>
        </section>

        <section className="site-frame home-sections">
          <section className="surface-grid">
            {surfacePanels.map((surface) => (
              <article key={surface.title} className="surface-panel">
                <p className="panel-kicker">{surface.title}</p>
                <p>{surface.body}</p>
              </article>
            ))}
          </section>

          <section className="deep-grid">
            <article className="deep-copy">
              <p className="section-tag">Supported today</p>
              <h2>Gunmetal ships a small, real provider set instead of a vague promise.</h2>
              <p>
                The current super app supports subscription providers, gateway
                providers, and direct OpenAI keys under the same local surface.
                The public rule is simple: if Gunmetal ships it today, the site
                should say so plainly.
              </p>
            </article>
            <div className="deep-stack">
              {providerLanes.map((band) => (
                <article key={band.title} className="band-panel">
                  <p className="panel-kicker">{band.title}</p>
                  <h3>{band.items.join(", ")}</h3>
                  <p>{band.body}</p>
                </article>
              ))}
            </div>
          </section>

          <section className="dual-panel">
            <article className="code-panel home-code-panel">
              <div className="code-head">
                <span>Golden path</span>
                <span>shell</span>
              </div>
              <pre>{installSnippet}</pre>
            </article>
            <article className="warning-band home-warning-band">
              <p className="section-tag">Compatibility rule</p>
              <h2>Gunmetal works when the app talks to Gunmetal.</h2>
              <p>
                A compatible app lets you set a custom base URL, send a custom
                API key, and choose an explicit model id like
                `openrouter/openai/gpt-5.1` or `codex/gpt-5.4`. If the app
                hardcodes an upstream endpoint, Gunmetal is not in the path and
                cannot help there.
              </p>
              <div className="button-row">
                <Link className="button button-secondary" href="/docs">
                  Read the API contract
                </Link>
                <Link className="button button-secondary" href="/webui">
                  See the local browser UI
                </Link>
              </div>
            </article>
          </section>
        </section>
      </main>

      <footer className="site-frame site-footer site-footer-home">
        <span>Hosted front door at gunmetalapp.vercel.app.</span>
        <span>One install path. One local API. One super app.</span>
      </footer>
    </div>
  );
}
