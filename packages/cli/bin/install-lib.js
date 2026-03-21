import { createHash } from "node:crypto";
import { createWriteStream, existsSync, readFileSync } from "node:fs";
import { chmod, mkdir, rename, rm, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import { pipeline } from "node:stream/promises";

const REPO = "Dhruv2mars/gunmetal";

export function binNameForPlatform(platform = process.platform) {
  return platform === "win32" ? "gunmetal.exe" : "gunmetal";
}

export function assetNameFor(platform = process.platform, arch = process.arch) {
  const ext = platform === "win32" ? ".exe" : "";
  return `gunmetal-${platform}-${arch}${ext}`;
}

export function checksumsAssetNameFor(platform = process.platform, arch = process.arch) {
  return `checksums-${platform}-${arch}.txt`;
}

export function resolveInstallRoot(env = process.env, home = homedir()) {
  return env.GUNMETAL_INSTALL_ROOT || join(home, ".gunmetal");
}

export function resolveInstallMetaPath(env = process.env, home = homedir()) {
  return join(resolveInstallRoot(env, home), "install-meta.json");
}

export function resolveInstalledBin(env = process.env, platform = process.platform, home = homedir()) {
  return join(resolveInstallRoot(env, home), "bin", binNameForPlatform(platform));
}

export function packageManagerHintFromEnv(env = process.env) {
  const execPath = String(env.npm_execpath || "").toLowerCase();
  if (execPath.includes("bun")) return "bun";
  if (execPath.includes("pnpm")) return "pnpm";
  if (execPath.includes("yarn")) return "yarn";
  if (execPath.includes("npm")) return "npm";

  const ua = String(env.npm_config_user_agent || "").toLowerCase();
  if (ua.startsWith("bun/")) return "bun";
  if (ua.startsWith("pnpm/")) return "pnpm";
  if (ua.startsWith("yarn/")) return "yarn";
  if (ua.startsWith("npm/")) return "npm";

  return null;
}

export function shouldInstallBinary({ binExists, installedVersion, packageVersion }) {
  if (!binExists) return true;
  if (!packageVersion) return false;
  return installedVersion !== packageVersion;
}

export function resolvePackageVersion(packageJsonPath, env = process.env) {
  try {
    const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
    return typeof pkg.version === "string" && pkg.version.length > 0
      ? pkg.version
      : (env.npm_package_version || "0.0.0");
  } catch {
    return env.npm_package_version || "0.0.0";
  }
}

export function parseChecksumForAsset(text, asset) {
  if (typeof text !== "string") return null;
  for (const line of text.split(/\r?\n/)) {
    const match = line.trim().match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (!match) continue;
    if (match[2].trim() !== asset) continue;
    return match[1].toLowerCase();
  }
  return null;
}

export async function installRuntime({
  version,
  env = process.env,
  platform = process.platform,
  arch = process.arch,
  home = homedir()
}) {
  const installRoot = resolveInstallRoot(env, home);
  const installBin = resolveInstalledBin(env, platform, home);
  const installMeta = resolveInstallMetaPath(env, home);
  const asset = assetNameFor(platform, arch);
  const checksumsAsset = checksumsAssetNameFor(platform, arch);
  const baseUrl = env.GUNMETAL_RELEASE_BASE_URL
    || `https://github.com/${REPO}/releases/download/v${version}`;

  await mkdir(join(installRoot, "bin"), { recursive: true });

  const checksumsResponse = await fetch(`${baseUrl}/${checksumsAsset}`);
  if (!checksumsResponse.ok) {
    throw new Error(`failed_download:${checksumsAsset}`);
  }
  const checksumsText = await checksumsResponse.text();
  const expectedChecksum = parseChecksumForAsset(checksumsText, asset);
  if (!expectedChecksum) {
    throw new Error(`missing_checksum:${asset}`);
  }

  const assetResponse = await fetch(`${baseUrl}/${asset}`);
  if (!assetResponse.ok || !assetResponse.body) {
    throw new Error(`failed_download:${asset}`);
  }

  const tempPath = `${installBin}.download`;
  try {
    await pipeline(assetResponse.body, createWriteStream(tempPath));
    const actualChecksum = createHash("sha256").update(readFileSync(tempPath)).digest("hex");
    if (actualChecksum !== expectedChecksum) {
      throw new Error(`checksum_mismatch:${asset}`);
    }

    if (platform !== "win32") {
      await chmod(tempPath, 0o755);
    }

    await rm(installBin, { force: true });
    await rename(tempPath, installBin);
  } catch (error) {
    await rm(tempPath, { force: true });
    throw error;
  }
  if (platform !== "win32") {
    await chmod(installBin, 0o755);
  }

  await writeFile(
    installMeta,
    JSON.stringify(
      {
        packageManager: packageManagerHintFromEnv(env),
        version
      },
      null,
      2
    ),
  );

  return {
    asset,
    installBin,
    installRoot,
    version
  };
}

export function readInstalledVersion(env = process.env, home = homedir()) {
  const metaPath = resolveInstallMetaPath(env, home);
  if (!existsSync(metaPath)) return null;
  try {
    return JSON.parse(readFileSync(metaPath, "utf8")).version || null;
  } catch {
    return null;
  }
}
