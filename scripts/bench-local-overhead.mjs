#!/usr/bin/env node

import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";

const env = process.env;
const gunmetalBin = env.GUNMETAL_BIN || "target/debug/gunmetal";
const benchPort = Number(env.GUNMETAL_BENCH_PORT || String(46000 + Math.floor(Math.random() * 1000)));
const upstreamPort = Number(env.LOCAL_UPSTREAM_PORT || String(47000 + Math.floor(Math.random() * 1000)));
const runs = Number(env.BENCH_RUNS || "7");
const warmups = Number(env.BENCH_WARMUPS || "1");
const upstreamBaseUrl = `http://127.0.0.1:${upstreamPort}/v1`;
const localApiKey = "sk-local-bench";
const model = "fake-local-model";
const streamChunks = ["one ", "two ", "three"];
const streamFirstDelayMs = 45;
const streamChunkDelayMs = 35;
const nonStreamDelayMs = 140;

const home = await mkdtemp(join(tmpdir(), "gunmetal-local-bench-"));
const childEnv = { ...env, GUNMETAL_HOME: home };
const upstream = await startUpstream(upstreamPort);

try {
  await exec(
    [
      gunmetalBin,
      "profiles",
      "create",
      "--provider",
      "openai",
      "--name",
      "local-bench",
      "--base-url",
      upstreamBaseUrl,
      "--api-key",
      localApiKey,
    ],
    childEnv,
  );
  await exec([gunmetalBin, "models", "sync", "local-bench"], childEnv);
  const keyOutput = await exec(
    [gunmetalBin, "keys", "create", "--name", "local-bench", "--provider", "openai"],
    childEnv,
  );
  const secret = keyOutput.stdout.match(/secret:\s+(gm_[^\s]+)/)?.[1];
  if (!secret) {
    throw new Error("Failed to parse Gunmetal key.");
  }

  await exec([gunmetalBin, "start", "--port", String(benchPort)], childEnv);
  await waitForHealth(`http://127.0.0.1:${benchPort}/health`);

  for (let index = 0; index < warmups; index += 1) {
    await directNonStream();
    await gunmetalNonStream(secret);
    await directStream();
    await gunmetalStream(secret);
  }

  const samples = {
    direct_non_stream_ms: [],
    gunmetal_non_stream_ms: [],
    direct_stream_ttft_ms: [],
    gunmetal_stream_ttft_ms: [],
    direct_stream_total_ms: [],
    gunmetal_stream_total_ms: [],
  };

  for (let index = 0; index < runs; index += 1) {
    samples.direct_non_stream_ms.push(await timed(directNonStream));
    samples.gunmetal_non_stream_ms.push(await timed(() => gunmetalNonStream(secret)));

    const directStreamMetrics = await directStream();
    samples.direct_stream_ttft_ms.push(directStreamMetrics.ttft_ms);
    samples.direct_stream_total_ms.push(directStreamMetrics.total_ms);

    const gunmetalStreamMetrics = await gunmetalStream(secret);
    samples.gunmetal_stream_ttft_ms.push(gunmetalStreamMetrics.ttft_ms);
    samples.gunmetal_stream_total_ms.push(gunmetalStreamMetrics.total_ms);
  }

  const result = {
    runs,
    warmups,
    upstream_base_url: upstreamBaseUrl,
    model,
    non_stream: compare(samples.direct_non_stream_ms, samples.gunmetal_non_stream_ms),
    stream_ttft: compare(samples.direct_stream_ttft_ms, samples.gunmetal_stream_ttft_ms),
    stream_total: compare(samples.direct_stream_total_ms, samples.gunmetal_stream_total_ms),
  };

  console.log(JSON.stringify(result, null, 2));
} finally {
  await exec([gunmetalBin, "stop", "--port", String(benchPort)], childEnv, {
    allowFailure: true,
  });
  upstream.close();
  await rm(home, { recursive: true, force: true });
}

async function directNonStream() {
  const response = await fetch(`${upstreamBaseUrl}/chat/completions`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${localApiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model,
      messages: [{ role: "user", content: "reply with ok" }],
      stream: false,
    }),
  });
  if (!response.ok) {
    throw new Error(`Direct non-stream failed: ${response.status} ${await response.text()}`);
  }
  await response.text();
}

async function gunmetalNonStream(secret) {
  const response = await fetch(`http://127.0.0.1:${benchPort}/v1/chat/completions`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${secret}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model: `openai/${model}`,
      messages: [{ role: "user", content: "reply with ok" }],
      stream: false,
    }),
  });
  if (!response.ok) {
    throw new Error(`Gunmetal non-stream failed: ${response.status} ${await response.text()}`);
  }
  await response.text();
}

function directStream() {
  return streamMetrics(`${upstreamBaseUrl}/chat/completions`, {
    Authorization: `Bearer ${localApiKey}`,
    "Content-Type": "application/json",
  }, {
    model,
    messages: [{ role: "user", content: "reply with ok" }],
    stream: true,
  });
}

function gunmetalStream(secret) {
  return streamMetrics(`http://127.0.0.1:${benchPort}/v1/chat/completions`, {
    Authorization: `Bearer ${secret}`,
    "Content-Type": "application/json",
  }, {
    model: `openai/${model}`,
    messages: [{ role: "user", content: "reply with ok" }],
    stream: true,
  });
}

async function streamMetrics(url, headers, body) {
  const started = performance.now();
  const response = await fetch(url, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`Stream request failed: ${response.status} ${await response.text()}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let ttft = null;

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true }).replace(/\r\n/g, "\n");
    while (true) {
      const boundary = buffer.indexOf("\n\n");
      if (boundary === -1) {
        break;
      }
      const frame = buffer.slice(0, boundary);
      buffer = buffer.slice(boundary + 2);
      const data = frame
        .split("\n")
        .filter((line) => line.startsWith("data:"))
        .map((line) => line.slice(5).trimStart())
        .join("\n");

      if (!data) {
        continue;
      }
      if (data === "[DONE]") {
        return {
          ttft_ms: round(ttft ?? performance.now() - started),
          total_ms: round(performance.now() - started),
        };
      }

      const payload = JSON.parse(data);
      const content = payload.choices?.[0]?.delta?.content;
      if (content && ttft === null) {
        ttft = performance.now() - started;
      }
    }
  }

  throw new Error("Stream ended before [DONE].");
}

async function timed(fn) {
  const started = performance.now();
  await fn();
  return round(performance.now() - started);
}

function compare(directValues, gunmetalValues) {
  const directMedian = median(directValues);
  const gunmetalMedian = median(gunmetalValues);
  const overheadMs = gunmetalMedian - directMedian;
  return {
    direct: summarize(directValues),
    gunmetal: summarize(gunmetalValues),
    overhead_ms_median: round(overheadMs),
    overhead_pct_median: directMedian === 0 ? 0 : round((overheadMs / directMedian) * 100),
  };
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
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {}
    await sleep(150);
  }
  throw new Error(`Gunmetal never became healthy at ${url}`);
}

async function startUpstream(port) {
  const server = createServer(async (request, response) => {
    const url = new URL(request.url, `http://127.0.0.1:${port}`);

    if (request.method === "GET" && url.pathname === "/v1/models") {
      response.writeHead(200, { "Content-Type": "application/json" });
      response.end(
        JSON.stringify({
          object: "list",
          data: [{ id: model }],
        }),
      );
      return;
    }

    if (request.method === "POST" && url.pathname === "/v1/chat/completions") {
      const body = await readJson(request);
      const requestModel = body.model;
      if (body.stream) {
        response.writeHead(200, {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
          Connection: "keep-alive",
        });

        await sleep(streamFirstDelayMs);
        for (const chunk of streamChunks) {
          response.write(
            `data: ${JSON.stringify({
              model: requestModel,
              choices: [{ delta: { content: chunk } }],
            })}\n\n`,
          );
          await sleep(streamChunkDelayMs);
        }

        response.write(
          `data: ${JSON.stringify({
            model: requestModel,
            choices: [{ finish_reason: "stop" }],
            usage: { prompt_tokens: 5, completion_tokens: 3, total_tokens: 8 },
          })}\n\n`,
        );
        response.write("data: [DONE]\n\n");
        response.end();
        return;
      }

      await sleep(nonStreamDelayMs);
      response.writeHead(200, { "Content-Type": "application/json" });
      response.end(
        JSON.stringify({
          model: requestModel,
          choices: [
            {
              finish_reason: "stop",
              message: { content: streamChunks.join("") },
            },
          ],
          usage: { prompt_tokens: 5, completion_tokens: 3, total_tokens: 8 },
        }),
      );
      return;
    }

    response.writeHead(404);
    response.end("not found");
  });

  await new Promise((resolve, reject) => {
    server.on("error", reject);
    server.listen(port, "127.0.0.1", resolve);
  });
  return server;
}

async function readJson(request) {
  let body = "";
  for await (const chunk of request) {
    body += chunk.toString();
  }
  return JSON.parse(body || "{}");
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
      reject(new Error(`Command failed (${code}): ${command.join(" ")}\n${stdout}${stderr}`));
    });
  });
}
