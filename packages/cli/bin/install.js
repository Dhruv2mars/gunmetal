#!/usr/bin/env node
import { join } from "node:path";

import { resolvePackageBinDir } from "./gunmetal-lib.js";
import { installRuntime, resolvePackageVersion } from "./install-lib.js";

const here = resolvePackageBinDir(import.meta.url);
const version = resolvePackageVersion(join(here, "..", "package.json"));

try {
  await installRuntime({ version });
} catch (error) {
  console.error(`gunmetal: install failed: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}
