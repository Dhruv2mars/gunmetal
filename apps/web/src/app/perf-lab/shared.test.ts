import { describe, expect, test } from "bun:test";

import {
  extractAssistantText,
  median,
  normalizeBaseUrl,
  summarizeRuns,
} from "./shared";

describe("perf lab helpers", () => {
  test("extracts assistant text from string content", () => {
    expect(
      extractAssistantText({
        choices: [{ message: { content: "OK" } }],
      }),
    ).toBe("OK");
  });

  test("extracts assistant text from part arrays", () => {
    expect(
      extractAssistantText({
        choices: [
          {
            message: {
              content: [
                { type: "output_text", text: "O" },
                { type: "output_text", text: "K" },
              ],
            },
          },
        ],
      }),
    ).toBe("OK");
  });

  test("summarizes latency runs", () => {
    expect(summarizeRuns([9.25, 12.5, 10, 14.75])).toEqual({
      minMs: 9.25,
      medianMs: 11.25,
      maxMs: 14.75,
    });
    expect(median([2, 8, 5])).toBe(5);
    expect(normalizeBaseUrl("http://127.0.0.1:4684/v1///")).toBe(
      "http://127.0.0.1:4684/v1",
    );
  });
});
