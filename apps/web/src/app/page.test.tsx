import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./webui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("Gunmetal");
    expect(html).toContain("Turn the AI access you already pay for into one local API.");
    expect(html).toContain("gunmetal setup");
    expect(html).toContain("One install path. One local API. One super app.");
    expect(html).toContain("gunmetalapp.vercel.app");
    expect(html).toContain("codex, copilot");
    expect(html).toContain("openrouter, zen");
    expect(html).toContain("openai");
    expect(html).not.toContain("azure");
    expect(html).not.toContain("nvidia");
  });

  test("web ui page explains the local browser flow", () => {
    const html = renderToStaticMarkup(<WebUiPage />);

    expect(html).toContain("Web UI");
    expect(html).toContain("gunmetal web");
    expect(html).toContain("127.0.0.1");
    expect(html).toContain("open it in your browser");
  });
});
