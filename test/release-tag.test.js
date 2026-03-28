const test = require("node:test");
const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { readFileSync } = require("node:fs");

test("release tag script prints npm package version tag", () => {
  const version = JSON.parse(readFileSync("packages/npm/package.json", "utf8")).version;
  const output = execFileSync("node", ["scripts/release-tag.mjs", "--print"], {
    cwd: process.cwd(),
    encoding: "utf8"
  }).trim();

  assert.equal(output, `v${version}`);
});
