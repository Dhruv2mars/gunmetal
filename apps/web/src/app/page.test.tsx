import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./webui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("Gunmetal");
    expect(html).toContain("Local AI Inference Gateway");
    expect(html).toContain("middleman");
    expect(html).toContain("AI inference");
    expect(html).toContain("Turn your AI subscriptions");
    expect(html).toContain("Install Gunmetal");
    expect(html).toContain("Documentation");
    expect(html).toContain("How It Works");
    expect(html).toContain("Three steps to full control");
    expect(html).toContain("Mount Providers");
    expect(html).toContain("Mint Access");
    expect(html).toContain("Direct Apps");
    expect(html).toContain("Connect everything");
    expect(html).toContain("Docs");
    expect(html).toContain("Install");
    expect(html).toContain("Start Here");
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