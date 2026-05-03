import test from "node:test";
import assert from "node:assert/strict";

import {
  defaultProbe,
  detectInstalledPackageManager,
  resolveUpdateCommand,
  shouldRunUpdateCommand
} from "../bin/gunmetal-lib.js";
import {
  packageManagerHintFromEnv,
  SUPPORTED_PACKAGE_MANAGERS
} from "../bin/install-lib.js";

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

test("registry contains all supported package managers", () => {
  const expected = ["bun", "npm", "pnpm", "yarn"];
  const actual = Object.keys(SUPPORTED_PACKAGE_MANAGERS);
  assert.deepEqual(actual.sort(), expected.sort());
  for (const name of expected) {
    const config = SUPPORTED_PACKAGE_MANAGERS[name];
    assert.ok(config.execPathHint, `${name} missing execPathHint`);
    assert.ok(config.userAgentHint, `${name} missing userAgentHint`);
    assert.ok(Array.isArray(config.listGlobalArgs), `${name} missing listGlobalArgs`);
    assert.ok(Array.isArray(config.installGlobalArgs), `${name} missing installGlobalArgs`);
  }
});

test("detects each package manager from env via execPath", () => {
  for (const [name, config] of Object.entries(SUPPORTED_PACKAGE_MANAGERS)) {
    const env = { npm_execpath: `/usr/local/bin/${config.execPathHint}` };
    assert.equal(packageManagerHintFromEnv(env), name);
  }
});

test("detects each package manager from env via userAgent", () => {
  for (const [name, config] of Object.entries(SUPPORTED_PACKAGE_MANAGERS)) {
    const env = { npm_config_user_agent: `${config.userAgentHint}1.0` };
    assert.equal(packageManagerHintFromEnv(env), name);
  }
});

test("defaultProbe uses registry listGlobalArgs", () => {
  const calls = [];
  const fakeRunner = (command, args) => {
    calls.push({ command, args });
    return { status: 0, stdout: "" };
  };
  defaultProbe("npm", fakeRunner);
  assert.deepEqual(calls, [{ command: "npm", args: SUPPORTED_PACKAGE_MANAGERS.npm.listGlobalArgs }]);
});

test("resolveUpdateCommand produces correct commands for each manager", () => {
  for (const [name, config] of Object.entries(SUPPORTED_PACKAGE_MANAGERS)) {
    const probe = (cmd) => ({
      status: cmd === name ? 0 : 1,
      stdout: cmd === name ? "@dhruv2mars/gunmetal" : ""
    });
    const result = detectInstalledPackageManager(probe, name);
    assert.equal(result, name);
  }
});

test("resolveUpdateCommand includes package name", () => {
  const command = resolveUpdateCommand({
    npm_execpath: "/usr/local/bin/npm",
    npm_config_user_agent: "npm/10"
  });
  assert.ok(command.args.some((arg) => arg.includes("@dhruv2mars/gunmetal")));
});
