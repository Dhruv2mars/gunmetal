import { GlobalWindow } from "happy-dom";
import { afterEach, mock } from "bun:test";
import * as React from "react";

const window = new GlobalWindow();
(globalThis as any).window = window;
(globalThis as any).document = window.document;
(globalThis as any).HTMLElement = window.HTMLElement;

mock.module("next/image", () => {
  return {
    __esModule: true,
    default: function ImageMock({ src, alt, ...props }: any) {
      return React.createElement("img", { src, alt, ...props });
    },
  };
});

// Import testing-library AFTER document is available so screen binds correctly.
const { cleanup } = await import("@testing-library/react");
afterEach(() => {
  cleanup();
});
