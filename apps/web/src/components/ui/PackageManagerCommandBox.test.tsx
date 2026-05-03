import { describe, expect, test, jest } from "bun:test";
import { render, screen, fireEvent, act } from "@testing-library/react";

import { PackageManagerCommandBox } from "./PackageManagerCommandBox";

describe("PackageManagerCommandBox", () => {
  test("cycles through package managers", () => {
    jest.useFakeTimers();

    render(
      <PackageManagerCommandBox
        packageName="@dhruv2mars/gunmetal"
        tail="i -g"
        cycleMs={1000}
      />,
    );

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toContain("npm");

    act(() => {
      jest.advanceTimersByTime(1000);
    });
    expect(button.getAttribute("aria-label")).toContain("bun");

    act(() => {
      jest.advanceTimersByTime(1000);
    });
    expect(button.getAttribute("aria-label")).toContain("pnpm");

    jest.useRealTimers();
  });

  test("copies command to clipboard and shows copied state", async () => {
    let copiedText = "";
    const originalClipboard = navigator.clipboard;
    Object.defineProperty(navigator, "clipboard", {
      value: {
        writeText: async (text: string) => {
          copiedText = text;
        },
      },
      writable: true,
      configurable: true,
    });

    render(
      <PackageManagerCommandBox
        packageName="@dhruv2mars/gunmetal"
        tail="i -g"
        cycleMs={99999}
      />,
    );

    const button = screen.getByRole("button");

    fireEvent.click(button);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 10));
    });

    expect(copiedText).toBe("npm i -g @dhruv2mars/gunmetal");
    expect(screen.getAllByText("Copied").length).toBeGreaterThanOrEqual(1);

    Object.defineProperty(navigator, "clipboard", {
      value: originalClipboard,
      writable: true,
      configurable: true,
    });
  });
});
