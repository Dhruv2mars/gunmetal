import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import {
  packageManagerHintFromEnv,
  readInstalledVersion,
  resolveInstalledBin,
  shouldInstallBinary,
  SUPPORTED_PACKAGE_MANAGERS,
  PACKAGE_NAME
} from "./install-lib.js";

export { resolveInstalledBin, shouldInstallBinary };

export function resolvePackageBinDir(importMetaUrl) {
  return dirname(realpathSync(fileURLToPath(importMetaUrl)));
}

export function shouldRunUpdateCommand(args) {
  return Array.isArray(args) && args[0] === "update";
}

export function defaultProbe(command, runner = spawnSync) {
  const config = SUPPORTED_PACKAGE_MANAGERS[command];
  if (!config) return { status: 1, stdout: "" };
  try {
    const result = runner(command, config.listGlobalArgs, { encoding: "utf8", stdio: "pipe" });
    return {
      status: result.status ?? 1,
      stdout: String(result.stdout || "")
    };
  } catch {
    return { status: 1, stdout: "" };
  }
}

export function detectInstalledPackageManager(probe = defaultProbe, preferred = null) {
  const managers = Object.keys(SUPPORTED_PACKAGE_MANAGERS);
  const searchOrder = preferred && managers.includes(preferred)
    ? [preferred, ...managers.filter((value) => value !== preferred)]
    : managers;
  for (const command of searchOrder) {
    const result = probe(command);
    if (result.status !== 0) continue;
    if (result.stdout.includes("@dhruv2mars/gunmetal")) {
      return command;
    }
  }
  return null;
}

function updateArgsFor(manager) {
  const config = SUPPORTED_PACKAGE_MANAGERS[manager];
  return config ? config.installGlobalArgs : SUPPORTED_PACKAGE_MANAGERS.npm.installGlobalArgs;
}

export function resolveUpdateCommand(env = process.env) {
  const hinted = packageManagerHintFromEnv(env);
  const manager = detectInstalledPackageManager(defaultProbe, hinted) || hinted || "npm";

  if (manager === "npm") {
    const npmExecPath = env.npm_execpath;
    if (typeof npmExecPath === "string" && npmExecPath.endsWith(".js")) {
      return {
        args: [npmExecPath, ...updateArgsFor("npm")],
        command: process.execPath
      };
    }
  }

  return {
    args: updateArgsFor(manager),
    command: manager
  };
}

export function readPackageVersion() {
  try {
    const here = resolvePackageBinDir(import.meta.url);
    const pkg = JSON.parse(readFileSync(join(here, "..", "package.json"), "utf8"));
    return pkg.version;
  } catch {
    return "";
  }
}

export function installedVersion(env = process.env) {
  return readInstalledVersion(env);
}
