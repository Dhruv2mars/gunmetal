import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";

import { TextLink, CodeBlock, PageFrame } from "./MarketingPrimitives";

describe("MarketingPrimitives", () => {
  test("TextLink renders correct href and text for external link", () => {
    render(
      <TextLink href="https://example.com">External</TextLink>,
    );

    const link = screen.getByRole("link", { name: "External" });
    expect(link.getAttribute("href")).toBe("https://example.com");
    expect(link.getAttribute("target")).toBe("_blank");
  });

  test("TextLink renders correct href and text for internal link", () => {
    render(<TextLink href="/docs">Documentation</TextLink>);

    const link = screen.getByRole("link", { name: "Documentation" });
    expect(link.getAttribute("href")).toBe("/docs");
  });

  test("CodeBlock renders children", () => {
    render(<CodeBlock>{"npm install example"}</CodeBlock>);

    expect(screen.getByText("npm install example")).toBeDefined();
  });

  test("PageFrame renders children in main element", () => {
    render(
      <PageFrame>
        <h1>Page Title</h1>
      </PageFrame>,
    );

    const main = screen.getByRole("main");
    expect(main).toBeDefined();
    expect(screen.getByRole("heading", { name: "Page Title" })).toBeDefined();
  });
});
