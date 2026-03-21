import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

import { assetNameFor, checksumsAssetNameFor } from "../bin/install-lib.js";

const testDir = fileURLToPath(new URL(".", import.meta.url));
const packageRoot = join(testDir, "..");
const repoRoot = join(packageRoot, "..", "..");
const packageJsonPath = join(packageRoot, "package.json");
const releaseWorkflow = join(repoRoot, ".github", "workflows", "release.yml");

function parseReleaseAssets(workflowText) {
  const includeBlocks = workflowText.match(/-\s+os:[\s\S]*?(?=\n\s*-\s+os:|\n\s*runs-on:|\n\s*steps:)/g) || [];
  const assets = [];
  for (const block of includeBlocks) {
    const platform = block.match(/platform:\s*"?([a-z0-9]+)"?/i)?.[1];
    const arch = block.match(/arch:\s*"?([a-z0-9_]+)"?/i)?.[1];
    const ext = block.match(/ext:\s*"([^"]*)"|ext:\s*'([^']*)'/)?.slice(1).find(Boolean) ?? "";
    if (!platform || !arch) continue;
    assets.push({ platform, arch, ext, name: `gunmetal-${platform}-${arch}${ext}` });
  }
  return assets;
}

test("npm package keeps publish contract", () => {
  const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));

  assert.equal(packageJson.name, "@dhruv2mars/gunmetal");
  assert.equal(packageJson.publishConfig.access, "public");
  assert.equal(packageJson.scripts.postinstall, "node bin/install.js");
  assert.equal(packageJson.bin.gunmetal, "bin/gunmetal.js");
  assert.equal(packageJson.repository.directory, "packages/npm");
});

test("release workflow declares expected installer asset matrix", () => {
  const assets = parseReleaseAssets(readFileSync(releaseWorkflow, "utf8"));
  const names = new Set(assets.map((item) => item.name));

  assert.deepEqual(
    names,
    new Set([
      "gunmetal-linux-x64",
      "gunmetal-linux-arm64",
      "gunmetal-win32-x64.exe",
      "gunmetal-win32-arm64.exe",
      "gunmetal-darwin-arm64",
      "gunmetal-darwin-x64"
    ])
  );
});

test("installer asset naming agrees with release matrix", () => {
  const assets = parseReleaseAssets(readFileSync(releaseWorkflow, "utf8"));
  for (const asset of assets) {
    assert.equal(assetNameFor(asset.platform, asset.arch), asset.name);
    assert.equal(
      checksumsAssetNameFor(asset.platform, asset.arch),
      `checksums-${asset.platform}-${asset.arch}.txt`
    );
  }
});

test("release workflow keeps publish contract", () => {
  const text = readFileSync(releaseWorkflow, "utf8");

  assert.match(text, /tags:\s*\n\s*-\s*["']v\*["']/);
  assert.match(text, /workflow_dispatch:/);
  assert.match(text, /repository_dispatch:/);
  assert.match(text, /release-rerun/);
  assert.match(text, /id-token: write/);
  assert.match(text, /actions\/checkout@v5/);
  assert.match(text, /actions\/setup-node@v5/);
  assert.match(text, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*true/);
  assert.match(text, /for f in \.\/gunmetal-\*/);
  assert.match(text, /sed 's# \\\.\/# #'/);
  assert.doesNotMatch(text, /shasum -a 256 \*/);
  assert.doesNotMatch(text, /sha256sum \*/);
  assert.doesNotMatch(text, /shasum -a 256 -- \.\/\*/);
  assert.doesNotMatch(text, /sha256sum -- \.\/\*/);
  assert.match(text, /npm publish --(?:provenance --)?access public/);
});

test("cli entry scripts are executable", () => {
  for (const relativePath of ["bin/gunmetal.js", "bin/install.js"]) {
    const filePath = join(packageRoot, relativePath);
    if (process.platform === "win32") {
      const text = readFileSync(filePath, "utf8");
      assert.match(text, /^#!\/usr\/bin\/env node/m, `${relativePath} must keep its node shebang`);
      continue;
    }
    const mode = statSync(filePath).mode & 0o777;
    assert.notEqual(mode & 0o111, 0, `${relativePath} must be executable`);
  }
});
