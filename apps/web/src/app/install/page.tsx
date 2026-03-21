import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";
import { installSnippet, installSteps } from "@/lib/site-content";

export const metadata: Metadata = {
  title: "Install",
};

const releaseItems = [
  "npm install -g @dhruv2mars/gunmetal",
  "GitHub release binaries for macOS, Linux, and Windows",
  "One binary includes the service, CLI, and TUI",
];

export default function InstallPage() {
  return (
    <SiteShell
      eyebrow="Install"
      title="One install path. One local home. One operator flow."
      lede="Gunmetal ships as a first-class npm install with release binaries alongside it. The daemon is internal. The product is still one download."
    >
      <section className="section-grid">
        <div className="lane-grid">
          <article className="lane-card">
            <p className="panel-kicker">Distribution</p>
            <div className="chip-row">
              {releaseItems.map((item) => (
                <span key={item} className="chip chip-wide">
                  {item}
                </span>
              ))}
            </div>
          </article>
          <article className="lane-card">
            <p className="panel-kicker">Local home</p>
            <ul className="stack-list">
              <li>state database</li>
              <li>provider sessions</li>
              <li>Gunmetal keys</li>
              <li>runtime pid + daemon logs</li>
              <li>lightweight request history</li>
            </ul>
          </article>
        </div>
      </section>

      <section className="section-grid split-code">
        <div className="section-head">
          <p className="section-tag">Quickstart</p>
          <h2>Five moves to a working local gateway.</h2>
          <ol className="step-list">
            {installSteps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </div>
        <article className="code-panel">
          <div className="code-head">
            <span>shell</span>
          </div>
          <pre>{installSnippet}</pre>
        </article>
      </section>
    </SiteShell>
  );
}
