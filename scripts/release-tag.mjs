import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const args = new Set(process.argv.slice(2));
const npmPackage = JSON.parse(readFileSync(new URL("../packages/npm/package.json", import.meta.url), "utf8"));
const cargoToml = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const cargoVersion = cargoToml.match(/\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/)?.[1];

if (!cargoVersion) {
  throw new Error("Could not read workspace package version from Cargo.toml");
}

if (npmPackage.version !== cargoVersion) {
  throw new Error(`Version mismatch: npm=${npmPackage.version}, cargo=${cargoVersion}`);
}

const tag = `v${npmPackage.version}`;

if (args.has("--print")) {
  process.stdout.write(`${tag}\n`);
  process.exit(0);
}

execFileSync("git", ["tag", tag], { stdio: "inherit" });
execFileSync("git", ["push", "origin", tag], { stdio: "inherit" });
