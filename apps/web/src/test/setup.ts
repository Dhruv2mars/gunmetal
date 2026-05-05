import { GlobalWindow } from "happy-dom";
import { afterEach, mock } from "bun:test";
import * as React from "react";

const happyWindow = new GlobalWindow();
const testGlobal = globalThis as typeof globalThis & {
  window: Window & typeof globalThis;
  document: Document;
  HTMLElement: typeof HTMLElement;
};
testGlobal.window = happyWindow as unknown as Window & typeof globalThis;
testGlobal.document = happyWindow.document as unknown as Document;
testGlobal.HTMLElement = happyWindow.HTMLElement as unknown as typeof HTMLElement;

type ImageMockProps = React.ImgHTMLAttributes<HTMLImageElement> & {
  src: string;
  alt: string;
};

mock.module("next/image", () => {
  return {
    __esModule: true,
    default: function ImageMock({ src, alt, ...props }: ImageMockProps) {
      return React.createElement("img", { src, alt, ...props });
    },
  };
});

// Import testing-library AFTER document is available so screen binds correctly.
const { cleanup } = await import("@testing-library/react");
afterEach(() => {
  cleanup();
});
