import test from "node:test";
import assert from "node:assert/strict";

import {
  defaultProbe,
  detectInstalledPackageManager,
  resolveUpdateCommand,
  shouldRunUpdateCommand
} from "../bin/gunmetal-lib.js";

test("detects update command", () => {
  assert.equal(shouldRunUpdateCommand(["update"]), true);
  assert.equal(shouldRunUpdateCommand(["status"]), false);
});

test("detects installed package manager from probe", () => {
  const probe = (command) => ({
    status: 0,
    stdout: command === "bun" ? "@dhruv2mars/gunmetal" : ""
  });
  assert.equal(detectInstalledPackageManager(probe, null), "bun");
});

test("resolves update command", () => {
  const command = resolveUpdateCommand({
    npm_execpath: "/usr/local/bin/npm",
    npm_config_user_agent: "npm/10"
  });
  assert.ok(command.args.includes("@dhruv2mars/gunmetal@latest"));
});

test("default probe handles missing command", () => {
  const result = defaultProbe("definitely-not-a-real-command");
  assert.equal(result.status, 1);
});
