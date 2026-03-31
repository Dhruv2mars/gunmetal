#!/usr/bin/env node
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn, spawnSync } from "node:child_process";

import {
  assetNameFor,
  checksumsAssetNameFor,
} from "../packages/npm/bin/install-lib.js";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..");
const tempRoot = mkdtempSync(join(tmpdir(), "gunmetal-install-smoke-"));
const releaseDir = join(tempRoot, "release");
const packageDir = join(tempRoot, "package");
const globalPrefix = join(tempRoot, "global");
const installRoot = join(tempRoot, "install");
const appHome = join(tempRoot, "gunmetal-home");
const npmPackageRoot = join(repoRoot, "packages", "npm");
const builtBinary = join(
  repoRoot,
  "target",
  "debug",
  process.platform === "win32" ? "gunmetal.exe" : "gunmetal",
);
const assetName = assetNameFor();
const checksumName = checksumsAssetNameFor();
const assetPath = join(releaseDir, assetName);
const packageJsonPath = join(npmPackageRoot, "package.json");
const packageVersion = JSON.parse(readFileSync(packageJsonPath, "utf8")).version;
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

if (!existsSync(builtBinary)) {
  fail(`missing built binary at ${builtBinary}. run cargo build -p gunmetal first`);
}

mkdirSync(releaseDir, { recursive: true });
mkdirSync(packageDir, { recursive: true });
mkdirSync(globalPrefix, { recursive: true });
copyFileSync(builtBinary, assetPath);
writeFileSync(
  join(releaseDir, checksumName),
  `${sha256(assetPath)} *${assetName}\n`,
);

const packed = run(npmCommand, ["pack", "."], { cwd: npmPackageRoot, capture: true });
const tarball = lastNonEmptyLine(packed.stdout);
if (!tarball) {
  fail("npm pack did not return a tarball path");
}
run("tar", ["-xzf", tarball, "-C", packageDir], { cwd: npmPackageRoot });

const extractedRoot = join(packageDir, "package");
const installer = join(extractedRoot, "bin", "install.js");
const launcher = join(extractedRoot, "bin", "gunmetal.js");
const tarballPath = join(npmPackageRoot, tarball);
const smokeEnv = {
  ...process.env,
  GUNMETAL_INSTALL_ROOT: installRoot,
  GUNMETAL_HOME: appHome,
};
let globalLauncher = null;

const server = http.createServer((request, response) => {
  const url = new URL(request.url || "/", "http://127.0.0.1");
  const path = join(releaseDir, decodeURIComponent(url.pathname.slice(1)));
  if (!existsSync(path)) {
    response.writeHead(404);
    response.end("not found");
    return;
  }
  response.writeHead(200);
  response.end(readFileSync(path));
});

const upstream = http.createServer((request, response) => {
  const url = new URL(request.url || "/", "http://127.0.0.1");
  const auth = request.headers.authorization;
  if (auth !== "Bearer sk-openai-test") {
    response.writeHead(401, { "content-type": "application/json" });
    response.end(JSON.stringify({ error: { message: "bad key" } }));
    return;
  }

  if (request.method === "GET" && url.pathname === "/v1/models") {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({
      object: "list",
      data: [{ id: "gpt-5.1" }, { id: "gpt-4.1-mini" }]
    }));
    return;
  }

  if (request.method === "POST" && url.pathname === "/v1/chat/completions") {
    let body = "";
    request.setEncoding("utf8");
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const payload = JSON.parse(body || "{}");
      const prompt = JSON.stringify(payload.messages || payload.input || "");
      const content = prompt.includes("responses")
        ? "GUNMETAL_RESPONSES_SMOKE_OK"
        : "GUNMETAL_CHAT_SMOKE_OK";
      response.writeHead(200, { "content-type": "application/json" });
      response.end(JSON.stringify({
        model: payload.model || "gpt-5.1",
        choices: [{
          finish_reason: "stop",
          message: { content }
        }],
        usage: {
          prompt_tokens: 6,
          completion_tokens: 2,
          total_tokens: 8
        }
      }));
    });
    return;
  }

  response.writeHead(404, { "content-type": "application/json" });
  response.end(JSON.stringify({ error: { message: "not found" } }));
});

await new Promise((resolvePromise) => server.listen(0, "127.0.0.1", resolvePromise));
const { port } = server.address();
const releaseBaseUrl = `http://127.0.0.1:${port}`;
await new Promise((resolvePromise) => upstream.listen(0, "127.0.0.1", resolvePromise));
const { port: upstreamPort } = upstream.address();
const upstreamBaseUrl = `http://127.0.0.1:${upstreamPort}/v1`;

try {
  await runAsync(npmCommand, ["install", "-g", "--prefix", globalPrefix, tarballPath], {
    cwd: repoRoot,
    env: {
      ...smokeEnv,
      GUNMETAL_RELEASE_BASE_URL: releaseBaseUrl,
    },
  });

  await runAsync(process.execPath, [installer], {
    cwd: repoRoot,
    env: {
      ...smokeEnv,
      GUNMETAL_RELEASE_BASE_URL: releaseBaseUrl,
    },
  });

  const installedApp = join(
    installRoot,
    "bin",
    process.platform === "win32" ? "gunmetal.exe" : "gunmetal",
  );
  const metaPath = join(installRoot, "install-meta.json");

  if (!existsSync(installedApp)) fail(`missing installed app binary at ${installedApp}`);
  if (!existsSync(metaPath)) fail(`missing install metadata at ${metaPath}`);

  const meta = JSON.parse(readFileSync(metaPath, "utf8"));
  if (meta.version !== packageVersion) {
    fail(`expected installed version ${packageVersion}, got ${meta.version}`);
  }

  globalLauncher = process.platform === "win32"
    ? join(globalPrefix, "gunmetal.cmd")
    : join(globalPrefix, "bin", "gunmetal");

  if (!existsSync(globalLauncher)) fail(`missing global launcher at ${globalLauncher}`);

  run(globalLauncher, ["--help"], {
    cwd: repoRoot,
    env: {
      ...smokeEnv,
    },
  });

  await runAsync(process.execPath, [launcher, "--help"], {
    cwd: repoRoot,
    env: {
      ...smokeEnv,
    },
  });

  const setup = await runAsyncCapture(globalLauncher, [
    "setup",
    "--provider", "openai",
    "--name", "smoke-openai",
    "--base-url", upstreamBaseUrl,
    "--api-key", "sk-openai-test",
    "--key-name", "smoke-key",
  ], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  const setupText = `${setup.stdout}\n${setup.stderr || ""}`;
  const apiKey = setupText.match(/API key:\s+(gm_[A-Za-z0-9_]+)/)?.[1];
  const model = setupText.match(/First model:\s+([^\s]+)/)?.[1];
  if (!apiKey) fail(`setup did not print a Gunmetal key\n${setupText}`);
  if (!model) fail(`setup did not print a first model\n${setupText}`);
  if (!setupText.includes("What just happened")) fail("setup summary missing");
  if (!setupText.includes("What to do next")) fail("setup next-step guidance missing");

  const start = await runAsyncCapture(globalLauncher, ["start"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  if (!String(start.stdout).includes("Gunmetal is running.")) {
    fail(`start output was not user-friendly\n${start.stdout}\n${start.stderr || ""}`);
  }

  const status = await runAsyncCapture(globalLauncher, ["status"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  if (!String(status.stdout).includes("Base URL: http://127.0.0.1:4684/v1")) {
    fail(`status output missing base url\n${status.stdout}`);
  }

  const modelsResponse = await fetch("http://127.0.0.1:4684/v1/models", {
    headers: { Authorization: `Bearer ${apiKey}` },
  });
  const modelsJson = await modelsResponse.json();
  if (modelsResponse.status !== 200) fail(`models failed: ${JSON.stringify(modelsJson)}`);
  if (!Array.isArray(modelsJson.data) || modelsJson.data.length === 0) {
    fail("models endpoint returned no data");
  }

  const chatResponse = await fetch("http://127.0.0.1:4684/v1/chat/completions", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model,
      messages: [{ role: "user", content: "chat smoke" }],
    }),
  });
  const chatJson = await chatResponse.json();
  if (chatJson.choices?.[0]?.message?.content !== "GUNMETAL_CHAT_SMOKE_OK") {
    fail(`chat smoke failed: ${JSON.stringify(chatJson)}`);
  }

  const responsesResponse = await fetch("http://127.0.0.1:4684/v1/responses", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model,
      input: "responses smoke",
    }),
  });
  const responsesJson = await responsesResponse.json();
  if (responsesJson.output?.[0]?.content?.[0]?.text !== "GUNMETAL_RESPONSES_SMOKE_OK") {
    fail(`responses smoke failed: ${JSON.stringify(responsesJson)}`);
  }

  const logs = await runAsyncCapture(globalLauncher, ["logs", "list", "--limit", "5"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  const logsText = `${logs.stdout}\n${logs.stderr || ""}`;
  if (!logsText.includes("/v1/chat/completions") || !logsText.includes("/v1/responses")) {
    fail(`logs output missing request history\n${logsText}`);
  }

  await runAsyncCapture(globalLauncher, ["keys", "revoke", "smoke-key"], {
    cwd: repoRoot,
    env: smokeEnv,
  });

  const revokedResponse = await fetch("http://127.0.0.1:4684/v1/models", {
    headers: { Authorization: `Bearer ${apiKey}` },
  });
  const revokedJson = await revokedResponse.json();
  if (revokedResponse.status !== 401) fail("revoked key should fail");
  if (!/invalid/i.test(String(revokedJson.error?.message || ""))) {
    fail(`revoked key error unclear: ${JSON.stringify(revokedJson)}`);
  }

  await runAsyncCapture(globalLauncher, ["auth", "logout", "smoke-openai"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  const authStatus = await runAsyncCapture(globalLauncher, ["auth", "status", "smoke-openai"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
  const authText = `${authStatus.stdout}\n${authStatus.stderr || ""}`;
  if (!authText.includes("Next: update the saved API key")) {
    fail(`auth recovery guidance missing\n${authText}`);
  }

  await runAsyncCapture(globalLauncher, ["stop"], {
    cwd: repoRoot,
    env: smokeEnv,
  });
} finally {
  try {
    await runAsyncCapture(globalLauncher, ["stop"], {
      cwd: repoRoot,
      env: smokeEnv,
    });
  } catch {}
  await new Promise((resolvePromise, rejectPromise) =>
    server.close((error) => (error ? rejectPromise(error) : resolvePromise()))
  );
  await new Promise((resolvePromise, rejectPromise) =>
    upstream.close((error) => (error ? rejectPromise(error) : resolvePromise()))
  );
}

console.log(`install smoke ok: ${process.platform}/${process.arch}`);
cleanup();

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function lastNonEmptyLine(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .at(-1);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env || process.env,
    shell: process.platform === "win32" && /\.cmd$/i.test(command),
    stdio: options.capture ? "pipe" : "inherit",
  });
  if (result.status !== 0) {
    fail(
      `${command} ${args.join(" ")} failed with code ${result.status ?? 1}\n${
        result.stderr || ""
      }`,
    );
  }
  return result;
}

function runAsync(command, args, options = {}) {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env || process.env,
      stdio: "inherit",
    });
    child.on("error", rejectPromise);
    child.on("exit", (code) => {
      if (code === 0) {
        resolvePromise();
        return;
      }
      rejectPromise(new Error(`${command} ${args.join(" ")} failed with code ${code ?? 1}`));
    });
  });
}

function runAsyncCapture(command, args, options = {}) {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env || process.env,
      shell: process.platform === "win32" && /\.cmd$/i.test(command),
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += String(chunk);
    });
    child.stderr.on("data", (chunk) => {
      stderr += String(chunk);
    });
    child.on("error", rejectPromise);
    child.on("exit", (code) => {
      if (code === 0) {
        resolvePromise({ stdout, stderr });
        return;
      }
      rejectPromise(new Error(`${command} ${args.join(" ")} failed with code ${code ?? 1}\n${stderr}`));
    });
  });
}

function cleanup() {
  try {
    rmSync(tempRoot, { force: true, recursive: true });
  } catch {}
}

function fail(message) {
  if (globalLauncher && existsSync(globalLauncher)) {
    try {
      run(globalLauncher, ["stop"], {
        cwd: repoRoot,
        env: smokeEnv,
        capture: true,
      });
    } catch {}
  }
  cleanup();
  console.error(`gunmetal install smoke: ${message}`);
  process.exit(1);
}
