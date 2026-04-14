import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./webui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("Gunmetal");
    expect(html).toContain("The middleman layer");
    expect(html).toContain("for AI inference.");
    expect(html).toContain("upstream providers");
    expect(html).toContain("i  -g  @dhruv2mars/gunmetal");
    expect(html).toContain("npm");
    expect(html).toContain("bun");
    expect(html).toContain("pnpm");
    expect(html).toContain("Core Philosophy");
    expect(html).toContain("Everything runs through a single, secure tunnel");
    expect(html).toContain("Local Keys Only");
    expect(html).toContain("Universal Protocol");
    expect(html).toContain("Native Performance");
    expect(html).toContain("Ready to build?");
    expect(html).toContain("GitHub");
  });

  test("web ui page explains the local browser flow", () => {
    const html = renderToStaticMarkup(<WebUiPage />);

    expect(html).toContain("Web UI");
    expect(html).toContain("gunmetal web");
    expect(html).toContain("127.0.0.1");
    expect(html).toContain("open it in your browser");
  });
});