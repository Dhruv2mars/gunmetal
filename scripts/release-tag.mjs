import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const args = new Set(process.argv.slice(2));
const npmPackage = JSON.parse(readFileSync(new URL("../packages/npm/package.json", import.meta.url), "utf8"));
const tag = `v${npmPackage.version}`;

if (args.has("--print")) {
  process.stdout.write(`${tag}\n`);
  process.exit(0);
}

execFileSync("git", ["tag", tag], { stdio: "inherit" });
execFileSync("git", ["push", "origin", tag], { stdio: "inherit" });
