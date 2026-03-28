#!/usr/bin/env node

import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";
import { spawn } from "node:child_process";

const env = process.env;
const gunmetalBin = env.GUNMETAL_BIN || "gunmetal";
const apiKey = env.ZEN_API_KEY || env.OPENCODE_API_KEY;
const zenBaseUrl = env.ZEN_BASE_URL || "https://opencode.ai/zen/v1";
const zenModel = env.ZEN_MODEL || "gpt-5.4";
const benchPort = Number(env.GUNMETAL_BENCH_PORT || "4689");
const runs = Number(env.BENCH_RUNS || "5");
const warmups = Number(env.BENCH_WARMUPS || "1");

if (!apiKey) {
  console.error("Missing ZEN_API_KEY or OPENCODE_API_KEY.");
  process.exit(1);
}

const home = await mkdtemp(join(tmpdir(), "gunmetal-bench-"));
const childEnv = { ...env, GUNMETAL_HOME: home };

try {
  await exec([gunmetalBin, "profiles", "create", "--provider", "zen", "--name", "bench", "--api-key", apiKey], childEnv);
  await exec([gunmetalBin, "models", "sync", "bench"], childEnv);
  const keyOutput = await exec(
    [gunmetalBin, "keys", "create", "--name", "bench-key", "--provider", "zen"],
    childEnv,
  );
  const secret = keyOutput.stdout.match(/secret:\s+(gm_[^\s]+)/)?.[1];
  if (!secret) {
    throw new Error("Failed to parse Gunmetal key secret.");
  }

  await exec([gunmetalBin, "start", "--port", String(benchPort)], childEnv);
  await waitForHealth(`http://127.0.0.1:${benchPort}/health`);

  for (let index = 0; index < warmups; index += 1) {
    await directRequest();
    await gunmetalRequest(secret);
  }

  const directTimes = [];
  const gunmetalTimes = [];
  for (let index = 0; index < runs; index += 1) {
    directTimes.push(await timed(directRequest));
    gunmetalTimes.push(await timed(() => gunmetalRequest(secret)));
  }

  const directMedian = median(directTimes);
  const gunmetalMedian = median(gunmetalTimes);
  const overheadMs = gunmetalMedian - directMedian;
  const overheadPct = directMedian === 0 ? 0 : (overheadMs / directMedian) * 100;

  console.log(
    JSON.stringify(
      {
        runs,
        warmups,
        zenBaseUrl,
        zenModel,
        direct: summarize(directTimes),
        gunmetal: summarize(gunmetalTimes),
        overhead_ms_median: round(overheadMs),
        overhead_pct_median: round(overheadPct),
      },
      null,
      2,
    ),
  );

  async function directRequest() {
    const response = await fetch(`${zenBaseUrl}/chat/completions`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: zenModel,
        messages: [{ role: "user", content: "reply with ok" }],
      }),
    });
    if (!response.ok) {
      throw new Error(`Direct Zen request failed: ${response.status} ${await response.text()}`);
    }
    await response.text();
  }

  async function gunmetalRequest(secretValue) {
    const response = await fetch(`http://127.0.0.1:${benchPort}/v1/chat/completions`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${secretValue}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: `zen/${zenModel}`,
        messages: [{ role: "user", content: "reply with ok" }],
      }),
    });
    if (!response.ok) {
      throw new Error(`Gunmetal request failed: ${response.status} ${await response.text()}`);
    }
    await response.text();
  }
} finally {
  await exec([gunmetalBin, "stop", "--port", String(benchPort)], childEnv, { allowFailure: true });
  await rm(home, { recursive: true, force: true });
}

async function timed(fn) {
  const started = performance.now();
  await fn();
  return performance.now() - started;
}

function summarize(values) {
  return {
    min_ms: round(Math.min(...values)),
    median_ms: round(median(values)),
    max_ms: round(Math.max(...values)),
  };
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0
    ? (sorted[middle - 1] + sorted[middle]) / 2
    : sorted[middle];
}

function round(value) {
  return Math.round(value * 100) / 100;
}

async function waitForHealth(url) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new Error(`Gunmetal never became healthy at ${url}`);
}

function exec(command, execEnv, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command[0], command.slice(1), {
      env: execEnv,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0 || options.allowFailure) {
        resolve({ stdout, stderr, code });
        return;
      }
      reject(
        new Error(
          `Command failed (${code}): ${command.join(" ")}\n${stdout}${stderr}`,
        ),
      );
    });
  });
}
