export const primaryNav = [
  { href: "/", label: "Overview" },
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
    body: "Handles auth, profiles, keys, models, logs, start, stop, and scripted workflows from one binary.",
  },
  {
    title: "TUI",
    body: "Runs as a real operator surface: create profiles, auth providers, sync models, mint or revoke keys, inspect snippets, and drill into request logs.",
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
    items: ["openai", "azure", "nvidia"],
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
  "Today, build Gunmetal locally or use a release binary. The npm wrapper exists in-repo but is not published yet.",
  "Run gunmetal to open the TUI. It auto-starts the local service when needed.",
  "Create provider profiles for codex, copilot, gateways, or direct-key providers.",
  "Create one or more Gunmetal keys for the local apps you want to point at Gunmetal.",
  "Set your app base URL to http://127.0.0.1:4684/v1 and use a Gunmetal model id.",
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
    body: "Wired codex, copilot, openrouter, zen, and openai into the same local control plane and inference path.",
  },
  {
    date: "March 2026",
    title: "Foundation",
    body: "Set the monorepo, local storage, Gunmetal keys, profiles, registry, lightweight telemetry, CLI, and TUI baseline.",
  },
];

export const installSnippet = `bun install
cargo run -p gunmetal --

# background service
cargo run -p gunmetal -- start
cargo run -p gunmetal -- status

# create provider profile
cargo run -p gunmetal -- profiles create --provider codex --name personal

# create local key
cargo run -p gunmetal -- keys create --name apps --scope inference,models_read --provider codex,copilot
`;

export const apiSnippet = `export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key

curl $OPENAI_BASE_URL/models \\
  -H "Authorization: Bearer $OPENAI_API_KEY"

curl $OPENAI_BASE_URL/chat/completions \\
  -H "Authorization: Bearer $OPENAI_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "codex/gpt-5.4",
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
