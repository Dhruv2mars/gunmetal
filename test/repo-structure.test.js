const test = require("node:test");
const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");

const expectedPaths = [
  "apps/cli/Cargo.toml",
  "apps/web/package.json",
  "packages/cli/Cargo.toml",
  "packages/core/Cargo.toml",
  "packages/daemon/Cargo.toml",
  "packages/npm/package.json",
  "packages/providers/Cargo.toml",
  "packages/storage/Cargo.toml",
  "packages/tui/Cargo.toml"
];

test("repo uses product-centric app and package roots", () => {
  for (const path of expectedPaths) {
    assert.equal(existsSync(path), true, `missing ${path}`);
  }

  assert.equal(existsSync("crates"), false, "legacy crates root should be gone");
});

test("workspace manifests point at the new layout", () => {
  const cargoToml = readFileSync("Cargo.toml", "utf8");
  const rootPackageJson = readFileSync("package.json", "utf8");
  const npmPackageJson = readFileSync("packages/npm/package.json", "utf8");

  for (const member of [
    "apps/cli",
    "packages/cli",
    "packages/core",
    "packages/daemon",
    "packages/providers",
    "packages/storage",
    "packages/tui"
  ]) {
    assert.match(cargoToml, new RegExp(`"${member}"`));
  }

  assert.match(rootPackageJson, /repo-structure\.test\.js/);
  assert.match(npmPackageJson, /"directory": "packages\/npm"/);
});
