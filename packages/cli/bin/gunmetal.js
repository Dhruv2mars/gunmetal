#!/usr/bin/env node
import { existsSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

import {
  installedVersion,
  readPackageVersion,
  resolveInstalledBin,
  resolvePackageBinDir,
  resolveUpdateCommand,
  shouldInstallBinary,
  shouldRunUpdateCommand
} from "./gunmetal-lib.js";

const args = process.argv.slice(2);

if (shouldRunUpdateCommand(args)) {
  const update = resolveUpdateCommand(process.env);
  const result = spawnSync(update.command, update.args, { stdio: "inherit", env: process.env });
  process.exit(result.status ?? 1);
}

if (process.env.GUNMETAL_BIN) {
  run(process.env.GUNMETAL_BIN, args);
}

const installedBin = resolveInstalledBin(process.env, process.platform);
const packageVersion = readPackageVersion();
const currentInstalledVersion = installedVersion(process.env);

if (shouldInstallBinary({
  binExists: existsSync(installedBin),
  installedVersion: currentInstalledVersion,
  packageVersion
})) {
  console.error("gunmetal: setting up native runtime...");
  const here = resolvePackageBinDir(import.meta.url);
  const installer = join(here, "install.js");
  const install = spawnSync(process.execPath, [installer], { stdio: "inherit", env: process.env });
  if (install.status !== 0 || !existsSync(installedBin)) {
    console.error("gunmetal: install missing. try reinstall: npm i -g @dhruv2mars/gunmetal");
    process.exit(1);
  }
}

run(installedBin, args);

function run(bin, binArgs) {
  const result = spawnSync(bin, binArgs, {
    stdio: "inherit",
    env: process.env
  });
  process.exit(result.status ?? 1);
}
