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

test("release workflow supports manual reruns and npm auth paths", () => {
  const releaseWorkflow = readFileSync(".github/workflows/release.yml", "utf8");

  assert.match(releaseWorkflow, /workflow_dispatch:/);
  assert.match(releaseWorkflow, /repository_dispatch:/);
  assert.match(releaseWorkflow, /release-rerun/);
  assert.match(releaseWorkflow, /release_tag:/);
  assert.match(releaseWorkflow, /id-token: write/);
  assert.match(
    releaseWorkflow,
    /trusted publisher not configured and NPM_TOKEN missing/
  );
});

test("install docs point at npm, not source-only fallback", () => {
  const rootReadme = readFileSync("README.md", "utf8");
  const npmReadme = readFileSync("packages/npm/README.md", "utf8");
  const siteContent = readFileSync("apps/web/src/lib/site-content.ts", "utf8");

  assert.doesNotMatch(rootReadme, /not published yet/);
  assert.doesNotMatch(rootReadme, /run Gunmetal from source/i);
  assert.match(rootReadme, /npm i -g @dhruv2mars\/gunmetal/);

  assert.doesNotMatch(npmReadme, /not published yet/);
  assert.match(npmReadme, /npm i -g @dhruv2mars\/gunmetal/);

  assert.doesNotMatch(siteContent, /not published yet/);
  assert.match(siteContent, /npm i -g @dhruv2mars\/gunmetal/);
});
