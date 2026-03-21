import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  assetNameFor,
  binNameForPlatform,
  checksumsAssetNameFor,
  installRuntime,
  packageManagerHintFromEnv,
  parseChecksumForAsset,
  resolveInstallRoot,
  resolveInstalledBin,
  shouldInstallBinary
} from "../bin/install-lib.js";

test("builds platform asset names", () => {
  assert.equal(assetNameFor("darwin", "arm64"), "gunmetal-darwin-arm64");
  assert.equal(assetNameFor("win32", "x64"), "gunmetal-win32-x64.exe");
  assert.equal(binNameForPlatform("win32"), "gunmetal.exe");
});

test("builds checksum asset names", () => {
  assert.equal(checksumsAssetNameFor("linux", "x64"), "checksums-linux-x64.txt");
});

test("resolves install root", () => {
  assert.equal(resolveInstallRoot({ GUNMETAL_INSTALL_ROOT: "/tmp/gm" }, "/home/test"), "/tmp/gm");
});

test("parses checksum lines", () => {
  const text = "abc123\n0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  gunmetal-darwin-arm64\n";
  assert.equal(
    parseChecksumForAsset(text, "gunmetal-darwin-arm64"),
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
  );
});

test("reads package manager hint and install need", () => {
  assert.equal(packageManagerHintFromEnv({ npm_config_user_agent: "bun/1.0" }), "bun");
  assert.equal(shouldInstallBinary({ binExists: false, installedVersion: null, packageVersion: "0.1.0" }), true);
  assert.equal(shouldInstallBinary({ binExists: true, installedVersion: "0.1.0", packageVersion: "0.1.0" }), false);
});

test("install runtime replaces temp download with final binary", async () => {
  const installRoot = await mkdtemp(join(tmpdir(), "gunmetal-install-test-"));
  const env = {
    GUNMETAL_INSTALL_ROOT: installRoot,
    GUNMETAL_RELEASE_BASE_URL: "https://example.test/releases/download/v0.1.0"
  };
  const asset = assetNameFor("darwin", "arm64");
  const body = Buffer.from("#!/bin/sh\necho gunmetal\n");
  const checksumHex = createHash("sha256").update(body).digest("hex");
  const originalFetch = globalThis.fetch;

  globalThis.fetch = async (url) => {
    if (String(url).endsWith(`/${checksumsAssetNameFor("darwin", "arm64")}`)) {
      return new Response(`${checksumHex}  ${asset}\n`);
    }
    if (String(url).endsWith(`/${asset}`)) {
      return new Response(body);
    }
    return new Response("missing", { status: 404 });
  };

  try {
    const result = await installRuntime({
      version: "0.1.0",
      env,
      platform: "darwin",
      arch: "arm64",
      home: installRoot
    });
    const installBin = resolveInstalledBin(env, "darwin", installRoot);
    assert.equal(result.installBin, installBin);
    assert.equal(existsSync(installBin), true);
    assert.equal(existsSync(`${installBin}.download`), false);
    assert.equal(readFileSync(installBin, "utf8"), body.toString("utf8"));
  } finally {
    globalThis.fetch = originalFetch;
    await rm(installRoot, { recursive: true, force: true });
  }
});
