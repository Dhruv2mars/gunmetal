const test = require("node:test");
const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");

const publicCrates = [
  {
    manifest: "packages/sdk-core/Cargo.toml",
    readme: "packages/sdk-core/README.md",
    name: "gunmetal-core",
    description: /shared SDK-facing types/i
  },
  {
    manifest: "packages/app-storage/Cargo.toml",
    readme: "packages/app-storage/README.md",
    name: "gunmetal-storage",
    description: /local state/i
  },
  {
    manifest: "packages/sdk/Cargo.toml",
    readme: "packages/sdk/README.md",
    name: "gunmetal-sdk",
    description: /provider adapter SDK/i
  },
  {
    manifest: "packages/extensions/Cargo.toml",
    readme: "packages/extensions/README.md",
    name: "gunmetal-providers",
    description: /first-party provider adapters/i
  }
];

test("provider SDK crates are publishable public packages", () => {
  for (const crate of publicCrates) {
    const manifest = readFileSync(crate.manifest, "utf8");

    assert.match(manifest, new RegExp(`name = "${crate.name}"`));
    assert.match(manifest, /description = "[^"]+"/);
    assert.match(manifest, crate.description);
    assert.match(manifest, /readme = "README\.md"/);
    assert.match(manifest, /documentation = "https:\/\/docs\.rs\//);
    assert.match(manifest, /keywords = \[/);
    assert.match(manifest, /categories = \[/);
    assert.doesNotMatch(manifest, /publish = false/);
    assert.equal(existsSync(crate.readme), true, `missing ${crate.readme}`);
  }
});

test("provider SDK docs expose the integration contract", () => {
  const rootReadme = readFileSync("README.md", "utf8");
  const webPage = readFileSync("apps/web/src/app/developer/sdk/page.tsx", "utf8");
  const sdkReadme = readFileSync("packages/sdk/README.md", "utf8");
  const providersReadme = readFileSync("packages/extensions/README.md", "utf8");
  const prompt = readFileSync("docs/prompt.md", "utf8");

  assert.match(rootReadme, /Gunmetal Provider SDK/);
  assert.match(rootReadme, /cargo add gunmetal-sdk gunmetal-core gunmetal-storage/);
  assert.match(webPage, /cargo add gunmetal-sdk gunmetal-core gunmetal-storage/);
  assert.match(sdkReadme, /impl ProviderAdapter/);
  assert.match(sdkReadme, /ProviderHub/);
  assert.match(sdkReadme, /AppPaths/);
  assert.match(providersReadme, /builtin_provider_hub/);
  assert.doesNotMatch(prompt, /Do not publish SDK packages/);
});
