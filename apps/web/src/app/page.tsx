import Link from "next/link";

const surfaces = [
  {
    title: "Landing page",
    body: "Hosted on Vercel at gunmetal.vercel.app with the install path, docs, and product story in one place.",
  },
  {
    title: "Web UI",
    body: "Run `gunmetal web` to open your local control plane in the browser on the same machine that owns the keys and sessions.",
  },
  {
    title: "CLI + TUI",
    body: "Use the same profiles, keys, logs, and models from a shell or from the terminal dashboard.",
  },
];

const proof = [
  "No hosted relay",
  "One local API surface",
  "Same state across browser, TUI, and CLI",
  "Explicit provider/model ids only",
];

const providerBands = [
  { label: "Subscription", items: "codex, copilot" },
  { label: "Gateway", items: "openrouter, zen" },
  { label: "Direct", items: "openai, azure, nvidia" },
];

const operators = [
  "Create or save provider profiles",
  "Finish auth flows for browser-login providers",
  "Sync upstream models into one local registry",
  "Mint scoped Gunmetal keys for real apps",
  "Inspect request logs and model inventory",
];

export default function HomePage() {
  return (
    <div className="home-page">
      <header className="site-frame site-header site-header-home">
        <Link className="brand-mark" href="/">
          <span className="brand-chip">GM</span>
          <span className="brand-copy">
            <strong>Gunmetal</strong>
            <span>Local-first AI switchboard</span>
          </span>
        </Link>
        <nav className="site-nav" aria-label="Primary">
          <Link href="/web-ui">Web UI</Link>
          <Link href="/install">Install</Link>
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
            <p className="eyebrow">Gunmetal • gunmetal.vercel.app</p>
            <h1>The local control plane for every AI account you already pay for.</h1>
            <p className="lede">
              Gunmetal puts one fast local API, one browser UI, one TUI, and
              one CLI in front of subscription providers, gateways, and direct
              keys without adding a hosted relay in the middle.
            </p>
            <div className="button-row">
              <Link className="button button-primary" href="/install">
                Install now
              </Link>
              <Link className="button button-secondary" href="/web-ui">
                See Web UI
              </Link>
            </div>
          </div>

          <div className="poster-stage">
            <div className="stage-window">
              <div className="stage-topline">
                <span>gunmetal web</span>
                <span>local browser UI</span>
              </div>
              <div className="stage-columns">
                <div>
                  <p className="panel-kicker">Operator flow</p>
                  <ul className="stack-list">
                    {operators.map((item) => (
                      <li key={item}>{item}</li>
                    ))}
                  </ul>
                </div>
                <div className="stage-command mono">
                  <span>npm i -g @dhruv2mars/gunmetal</span>
                  <span>gunmetal setup</span>
                  <span>gunmetal web</span>
                  <span>OPENAI_BASE_URL=http://127.0.0.1:4684/v1</span>
                </div>
              </div>
            </div>
            <div className="proof-strip">
              {proof.map((item) => (
                <span key={item}>{item}</span>
              ))}
            </div>
          </div>
        </section>

        <section className="site-frame home-sections">
          <section className="surface-grid">
            {surfaces.map((surface) => (
              <article key={surface.title} className="surface-panel">
                <p className="panel-kicker">{surface.title}</p>
                <p>{surface.body}</p>
              </article>
            ))}
          </section>

          <section className="deep-grid">
            <article className="deep-copy">
              <p className="section-tag">Why this exists</p>
              <h2>Apps should talk to one local endpoint. Providers stay explicit behind it.</h2>
              <p>
                Gunmetal is built for the boring real problem: you already have
                a mix of Codex, Copilot, OpenRouter, Zen, and direct provider
                keys, but each app wants a different setup story. Gunmetal
                gives your machine one consistent front door and keeps the
                provider choice visible instead of hiding it behind aliases.
              </p>
            </article>
            <div className="deep-stack">
              {providerBands.map((band) => (
                <article key={band.label} className="band-panel">
                  <p className="panel-kicker">{band.label}</p>
                  <h3>{band.items}</h3>
                </article>
              ))}
            </div>
          </section>

          <section className="dual-panel">
            <article className="code-panel home-code-panel">
              <div className="code-head">
                <span>First real path</span>
              </div>
              <pre>{`npm i -g @dhruv2mars/gunmetal

gunmetal setup
gunmetal web

# or keep the daemon alive without opening the browser
gunmetal start
gunmetal status`}</pre>
            </article>
            <article className="warning-band home-warning-band">
              <p className="section-tag">Compatibility rule</p>
              <h2>Gunmetal works when the app talks to Gunmetal.</h2>
              <p>
                Custom base URL. Custom API key. Arbitrary model names like
                `provider/model`. If the app hardcodes the upstream endpoint,
                Gunmetal cannot help there.
              </p>
              <div className="button-row">
                <Link className="button button-secondary" href="/docs">
                  Read the API contract
                </Link>
                <Link className="button button-secondary" href="/start-here">
                  Follow the setup path
                </Link>
              </div>
            </article>
          </section>
        </section>
      </main>

      <footer className="site-frame site-footer site-footer-home">
        <span>Hosted front door at gunmetal.vercel.app.</span>
        <span>Browser UI, TUI, CLI, and local API share one state.</span>
      </footer>
    </div>
  );
}
