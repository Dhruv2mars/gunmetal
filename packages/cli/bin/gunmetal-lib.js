import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import {
  packageManagerHintFromEnv,
  readInstalledVersion,
  resolveInstalledBin,
  shouldInstallBinary
} from "./install-lib.js";

const PACKAGE_NAME = "@dhruv2mars/gunmetal@latest";
const SUPPORTED_PACKAGE_MANAGERS = new Set(["bun", "npm", "pnpm", "yarn"]);

export { resolveInstalledBin, shouldInstallBinary };

export function resolvePackageBinDir(importMetaUrl) {
  return dirname(realpathSync(fileURLToPath(importMetaUrl)));
}

export function shouldRunUpdateCommand(args) {
  return Array.isArray(args) && args[0] === "update";
}

export function defaultProbe(command, runner = spawnSync) {
  const args = command === "bun"
    ? ["pm", "ls", "-g"]
    : command === "pnpm"
      ? ["list", "-g", "--depth=0"]
      : command === "yarn"
        ? ["global", "list", "--depth=0"]
        : ["list", "-g", "--depth=0"];
  try {
    const result = runner(command, args, { encoding: "utf8", stdio: "pipe" });
    return {
      status: result.status ?? 1,
      stdout: String(result.stdout || "")
    };
  } catch {
    return { status: 1, stdout: "" };
  }
}

export function detectInstalledPackageManager(probe = defaultProbe, preferred = null) {
  const searchOrder = preferred && SUPPORTED_PACKAGE_MANAGERS.has(preferred)
    ? [preferred, ...[...SUPPORTED_PACKAGE_MANAGERS].filter((value) => value !== preferred)]
    : [...SUPPORTED_PACKAGE_MANAGERS];
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
  if (manager === "bun") return ["add", "-g", PACKAGE_NAME];
  if (manager === "pnpm") return ["add", "-g", PACKAGE_NAME];
  if (manager === "yarn") return ["global", "add", PACKAGE_NAME];
  return ["install", "-g", PACKAGE_NAME];
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
