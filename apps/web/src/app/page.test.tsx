import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./webui/page";
import DocsPage from "./docs/page";
import DownloadPage from "./download/page";
import ProductSuitePage from "./products/suite/page";
import DeveloperSdkPage from "./developer/sdk/page";
import { normalizeGitHubRelease } from "./changelogs/releases";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("The middleman layer");
    expect(html).toContain("for AI inference.");
    expect(html).toContain("upstream providers");
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

  test("docs page keeps the compact setup path", () => {
    const html = renderToStaticMarkup(<DocsPage />);

    expect(html).toContain("Documentation");
    expect(html).toContain("Quick start");
    expect(html).toContain("gunmetal setup");
    expect(html).toContain("/v1/chat/completions");
  });

  test("download page offers the install path", () => {
    const html = renderToStaticMarkup(<DownloadPage />);

    expect(html).toContain("Install Gunmetal");
    expect(html).toContain("@dhruv2mars/gunmetal");
    expect(html).toContain("gunmetal setup");
    expect(html).toContain("GitHub releases");
  });

  test("product and developer pages fill navbar destinations", () => {
    const product = renderToStaticMarkup(<ProductSuitePage />);
    const developer = renderToStaticMarkup(<DeveloperSdkPage />);

    expect(product).toContain("Gunmetal Suite");
    expect(product).toContain("provider extension");
    expect(developer).toContain("Extension SDK");
    expect(developer).toContain("packages/extensions");
  });

  test("changelog release normalizer follows GitHub releases", () => {
    const release = normalizeGitHubRelease({
      body: "feat: ship page",
      draft: false,
      html_url: "https://github.com/Dhruv2mars/gunmetal/releases/tag/v1",
      name: "v1",
      prerelease: false,
      published_at: "2026-04-24T00:00:00Z",
      tag_name: "v1",
    });

    expect(release.title).toBe("v1");
    expect(release.body).toContain("ship page");
    expect(release.url).toContain("github.com/Dhruv2mars/gunmetal");
  });
});
