import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "./page";
import WebUiPage from "./web-ui/page";

describe("marketing routes", () => {
  test("home page sells the full product", () => {
    const html = renderToStaticMarkup(<HomePage />);

    expect(html).toContain("Gunmetal");
    expect(html).toContain("gunmetal.vercel.app");
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
});
