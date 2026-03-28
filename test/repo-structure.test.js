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

  assert.match(rootPackageJson, /test\/\*\.test\.js/);
  assert.match(npmPackageJson, /"directory": "packages\/npm"/);
});

test("release workflow supports manual reruns and npm auth paths", () => {
  const releaseWorkflow = readFileSync(".github/workflows/release.yml", "utf8");
  const ciWorkflow = readFileSync(".github/workflows/ci.yml", "utf8");

  assert.match(releaseWorkflow, /workflow_dispatch:/);
  assert.match(releaseWorkflow, /repository_dispatch:/);
  assert.match(releaseWorkflow, /release-rerun/);
  assert.match(releaseWorkflow, /release_tag:/);
  assert.match(releaseWorkflow, /id-token: write/);
  assert.match(releaseWorkflow, /GITHUB_EVENT_PATH/);
  assert.match(releaseWorkflow, /npm publish --(?:provenance --)?access public/);
  assert.doesNotMatch(
    releaseWorkflow,
    /RELEASE_TAG:\s+\$\{\{\s*github\.event\.client_payload/
  );
  assert.match(ciWorkflow, /actionlint/);
  assert.match(ciWorkflow, /rhysd\/actionlint@v1\.7\.11/);
  assert.doesNotMatch(ciWorkflow, /rhysd\/actionlint@v1(\s|$)/);
});

test("install docs point at npm, not source-only fallback", () => {
  const rootReadme = readFileSync("README.md", "utf8");
  const npmReadme = readFileSync("packages/npm/README.md", "utf8");
  const siteContent = readFileSync("apps/web/src/lib/site-content.ts", "utf8");
  const installPage = readFileSync("apps/web/src/app/install/page.tsx", "utf8");
  const docsPage = readFileSync("apps/web/src/app/docs/page.tsx", "utf8");
  const startHerePage = readFileSync("apps/web/src/app/start-here/page.tsx", "utf8");

  assert.doesNotMatch(rootReadme, /not published yet/);
  assert.doesNotMatch(rootReadme, /run Gunmetal from source/i);
  assert.match(rootReadme, /npm i -g @dhruv2mars\/gunmetal/);
  assert.match(rootReadme, /gunmetal setup/);
  assert.match(rootReadme, /curl .*\/v1\/models/s);

  assert.doesNotMatch(npmReadme, /not published yet/);
  assert.match(npmReadme, /npm i -g @dhruv2mars\/gunmetal/);
  assert.match(npmReadme, /gunmetal setup/);
  assert.match(npmReadme, /Gunmetal works when the app talks to Gunmetal/i);

  assert.doesNotMatch(siteContent, /not published yet/);
  assert.match(siteContent, /npm i -g @dhruv2mars\/gunmetal/);
  assert.match(siteContent, /gunmetal setup/);
  assert.match(siteContent, /OpenAI-compatible/);

  assert.doesNotMatch(installPage, /npm install -g gunmetal/);
  assert.match(installPage, /@dhruv2mars\/gunmetal/);
  assert.match(docsPage, /compatibility/i);
  assert.match(startHerePage, /Start here/i);
  assert.match(startHerePage, /\/v1\/models/);
  assert.match(startHerePage, /\/v1\/chat\/completions/);
});

test("web app pins Vercel to the Next.js build path", () => {
  const vercelConfig = readFileSync("apps/web/vercel.json", "utf8");

  assert.match(vercelConfig, /"framework":\s*"nextjs"/);
  assert.match(vercelConfig, /"installCommand":\s*"bun install --frozen-lockfile"/);
  assert.match(vercelConfig, /"buildCommand":\s*"bun run build"/);
});

test("workspace cargo version matches the published npm package version", () => {
  const cargoToml = readFileSync("Cargo.toml", "utf8");
  const npmPackageJson = JSON.parse(readFileSync("packages/npm/package.json", "utf8"));
  const cargoVersion = cargoToml.match(/^\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m)?.[1];

  assert.equal(cargoVersion, npmPackageJson.version);
});
