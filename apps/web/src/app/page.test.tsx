import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./webui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("One local API");
    expect(html).toContain("OpenAI-compatible endpoint");
    expect(html).toContain("provider/model");
    expect(html).toContain("i -g @dhruv2mars/gunmetal");
    expect(html).toContain("@dhruv2mars/gunmetal");
    expect(html).toContain("npm");
    expect(html).toContain("bun");
    expect(html).toContain("pnpm");
  });

  test("web ui page explains the local browser flow", () => {
    const html = renderToStaticMarkup(<WebUiPage />);

    expect(html).toContain("Web UI");
    expect(html).toContain("gunmetal web");
    expect(html).toContain("127.0.0.1");
    expect(html).toContain("Browser control");
  });
});
