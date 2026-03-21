import { createHash } from "node:crypto";
import { createWriteStream, existsSync, readFileSync } from "node:fs";
import { chmod, mkdir, rename, rm, writeFile } from "node:fs/promises";
import http from "node:http";
import https from "node:https";
import { homedir } from "node:os";
import { join } from "node:path";

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

function requestProtocolFor(url) {
  return new URL(url).protocol;
}

export function requestText(url, redirects = 0) {
  if (redirects > 5) {
    throw new Error("too_many_redirects");
  }
  return new Promise((resolve, reject) => {
    const transport = requestProtocolFor(url) === "http:" ? http : https;
    const request = transport.get(
      url,
      {
        agent: false,
        headers: {
          Connection: "close",
          "User-Agent": "gunmetal-installer"
        }
      },
      (response) => {
        if (
          response.statusCode
          && response.statusCode >= 300
          && response.statusCode < 400
          && response.headers.location
        ) {
          response.resume();
          requestText(response.headers.location, redirects + 1).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`http ${response.statusCode}`));
          return;
        }
        let data = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () => resolve(data));
      }
    );
    request.on("error", reject);
  });
}

export function download(url, outputPath, redirects = 0) {
  if (redirects > 5) {
    throw new Error("too_many_redirects");
  }
  const partPath = `${outputPath}.part`;
  return new Promise((resolve, reject) => {
    const transport = requestProtocolFor(url) === "http:" ? http : https;
    const request = transport.get(
      url,
      {
        agent: false,
        headers: {
          Connection: "close",
          "User-Agent": "gunmetal-installer"
        }
      },
      (response) => {
        if (
          response.statusCode
          && response.statusCode >= 300
          && response.statusCode < 400
          && response.headers.location
        ) {
          response.resume();
          download(response.headers.location, outputPath, redirects + 1).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`http ${response.statusCode}`));
          return;
        }
        const file = createWriteStream(partPath);
        file.on("error", async (error) => {
          await rm(partPath, { force: true });
          reject(error);
        });
        response.on("error", async (error) => {
          await rm(partPath, { force: true });
          reject(error);
        });
        response.pipe(file);
        file.on("finish", () => {
          file.close(async () => {
            try {
              await rename(partPath, outputPath);
              resolve();
            } catch (error) {
              await rm(partPath, { force: true });
              reject(error);
            }
          });
        });
      }
    );
    request.on("error", async (error) => {
      await rm(partPath, { force: true });
      reject(error);
    });
  });
}

export async function installRuntime({
  version,
  env = process.env,
  platform = process.platform,
  arch = process.arch,
  home = homedir(),
  downloadFn = download,
  requestTextFn = requestText
}) {
  const installRoot = resolveInstallRoot(env, home);
  const installBin = resolveInstalledBin(env, platform, home);
  const installMeta = resolveInstallMetaPath(env, home);
  const asset = assetNameFor(platform, arch);
  const checksumsAsset = checksumsAssetNameFor(platform, arch);
  const baseUrl = env.GUNMETAL_RELEASE_BASE_URL
    || `https://github.com/${REPO}/releases/download/v${version}`;

  await mkdir(join(installRoot, "bin"), { recursive: true });

  let checksumsText;
  try {
    checksumsText = await requestTextFn(`${baseUrl}/${checksumsAsset}`);
  } catch {
    throw new Error(`failed_download:${checksumsAsset}`);
  }
  const expectedChecksum = parseChecksumForAsset(checksumsText, asset);
  if (!expectedChecksum) {
    throw new Error(`missing_checksum:${asset}`);
  }

  const tempPath = `${installBin}.download`;
  try {
    try {
      await downloadFn(`${baseUrl}/${asset}`, tempPath);
    } catch {
      throw new Error(`failed_download:${asset}`);
    }
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
