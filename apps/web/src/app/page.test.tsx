import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import PerfLabPage from "./perf-lab/page";
import WebUiPage from "./web-ui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("Gunmetal");
    expect(html).toContain("gunmetalapp.vercel.app");
    expect(html).toContain("gunmetal web");
    expect(html).toContain("No hosted relay");
  });

  test("web ui page explains the local browser flow", () => {
    const html = renderToStaticMarkup(<WebUiPage />);

    expect(html).toContain("Web UI");
    expect(html).toContain("gunmetal web");
    expect(html).toContain("127.0.0.1");
    expect(html).toContain("open it in your browser");
  });

  test("perf lab page exposes the browser benchmark surface", () => {
    const html = renderToStaticMarkup(<PerfLabPage />);

    expect(html).toContain("Perf Lab");
    expect(html).toContain("openai-compatible");
    expect(html).toContain("OpenRouter");
    expect(html).toContain("Codex");
  });
});
