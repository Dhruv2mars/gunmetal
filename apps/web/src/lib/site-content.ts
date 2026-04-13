export const primaryNav = [
  { href: "/", label: "Home" },
  { href: "/webui", label: "Web UI" },
  { href: "/start-here", label: "Start Here" },
  { href: "/docs", label: "Docs" },
  { href: "/install", label: "Install" },
  { href: "/changelog", label: "Changelog" },
];

export const moduleCards = [
  {
    title: "Daemon + Service",
    body: "Owns the local API, autostart, port binding, health, and request flow. Other apps talk to this, not your upstream accounts.",
  },
  {
    title: "CLI",
    body: "Handles provider auth, keys, models, request history, start, stop, and scripted workflows from one binary.",
  },
  {
    title: "TUI",
    body: "Runs as a real operator surface: connect providers, auth them, sync models, mint or revoke keys, inspect snippets, and drill into request history.",
  },
  {
    title: "Providers",
    body: "Explicit adapters for subscription providers, gateways, and direct API-key providers with strict model prefixes.",
  },
  {
    title: "Keys + Auth",
    body: "Creates Gunmetal-local keys with scopes, provider limits, expiry, disable, revoke, and local-file auth storage.",
  },
  {
    title: "Registry + Telemetry",
    body: "Keeps a unified model catalog and lightweight local request history by default, with payload logging left opt-in.",
  },
];

export const providerLanes = [
  {
    title: "Subscription",
    body: "Bring the plans you already pay for.",
    items: ["codex", "copilot"],
  },
  {
    title: "Gateways",
    body: "Route through hosted model routers when you want breadth.",
    items: ["openrouter", "zen"],
  },
  {
    title: "Direct API Key",
    body: "Use standard provider keys under the same local surface.",
    items: ["openai"],
  },
];

export const docsSections = [
  {
    title: "Core contract",
    body: "Gunmetal issues Gunmetal keys. They work only because requests go to Gunmetal. The upstream provider never sees a fake Gunmetal-issued OpenAI or Copilot key.",
  },
  {
    title: "Gateway semantics",
    body: "Gunmetal is a normalized gateway by default. Use passthrough only when you explicitly request provider-native behavior through gunmetal.mode and provider_options.",
  },
  {
    title: "Model naming",
    body: "Every model is provider-prefixed. Examples: codex/gpt-5.4, copilot/claude-sonnet-4.5, openrouter/openai/gpt-5.1. There are no alias modes and no policy routing.",
  },
  {
    title: "Compatibility rule",
    body: "Third-party apps work well when they let you set a custom base URL, send a custom API key, and choose an arbitrary model string. If an app hardcodes an upstream endpoint, Gunmetal cannot help there.",
  },
  {
    title: "Storage rule",
    body: "Everything stays in local app files. No OS keychain. No hosted relay. No shared cloud vault.",
  },
];

export const installSteps = [
  "Install Gunmetal with npm: npm i -g @dhruv2mars/gunmetal.",
  "Run gunmetal setup. That is the golden path: connect one provider, auth it, sync models, create one key.",
  "Run gunmetal start or open gunmetal. The TUI can drive the same flow if you prefer.",
  "Call GET /v1/models with your Gunmetal key to confirm the local gateway works.",
  "Point your app at http://127.0.0.1:4684/v1 and use a provider/model id like codex/gpt-5.4.",
];

export const changelogEntries = [
  {
    date: "March 21, 2026",
    title: "Product hardening",
    body: "Added real daemon autostart, start/stop/status flows, richer TUI state, local logs commands, and the full landing/docs/install/changelog site.",
  },
  {
    date: "March 2026",
    title: "API coverage",
    body: "Completed both chat/completions and responses support under one local Gunmetal contract.",
  },
  {
    date: "March 2026",
    title: "Core provider set",
    body: "Wired codex, copilot, openrouter, zen, and openai into the same local inference middle layer and request path.",
  },
  {
    date: "March 2026",
    title: "Foundation",
    body: "Set the monorepo, local storage, Gunmetal keys, provider records, registry, lightweight telemetry, CLI, and TUI baseline.",
  },
];

export const installSnippet = `npm i -g @dhruv2mars/gunmetal

gunmetal setup
gunmetal web

# or keep the daemon running without opening the browser
gunmetal start
gunmetal status

# inspect what setup created
gunmetal profiles list
gunmetal keys list
`;

export const apiSnippet = `export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key
export MODEL=codex/gpt-5.4

curl $OPENAI_BASE_URL/models \\
  -H "Authorization: Bearer $OPENAI_API_KEY"

curl $OPENAI_BASE_URL/chat/completions \\
  -H "Authorization: Bearer $OPENAI_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "'$MODEL'",
    "messages": [{"role":"user","content":"say ok"}]
  }'
`;

export const responsesSnippet = `curl http://127.0.0.1:4684/v1/responses \\
  -H "Authorization: Bearer gm_your_local_key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "openrouter/openai/gpt-5.1",
    "instructions": "be terse",
    "input": "summarize the last build",
    "gunmetal": { "mode": "passthrough" },
    "provider_options": { "reasoning": { "effort": "high" } }
  }'
`;

export const openAiCompatSnippet = `OPENAI_BASE_URL=http://127.0.0.1:4684/v1
OPENAI_API_KEY=gm_your_local_key
MODEL=openai/gpt-5.1

# use these three values in any OpenAI-compatible app
# Gunmetal works when the app talks to Gunmetal
`;
