# @dhruv2mars/gunmetal

Alpha local-first AI access as one OpenAI-compatible local API.

Gunmetal installs a native CLI that runs a local Dashboard and API at `http://127.0.0.1:4684/` and `http://127.0.0.1:4684/v1`.

> Alpha software: commands, provider behavior, SDK contracts, and dashboard flows can change while Gunmetal stabilizes.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

The package downloads the native `gunmetal` binary into `~/.gunmetal/bin/`.

## Quickstart

```bash
gunmetal setup
gunmetal start
gunmetal status
```

`gunmetal setup` connects one provider, checks auth, syncs models, creates one local Gunmetal key, and prints a test request.

`gunmetal start` opens the Dashboard and keeps the local API running. Use `gunmetal start --no-open` when you only want the daemon.

## Use With Apps

```bash
export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key
```

```bash
curl "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "provider/model",
    "messages": [{"role":"user","content":"say ok"}]
  }'
```

Gunmetal works when the app accepts:

- custom OpenAI-compatible base URL
- custom Gunmetal API key
- provider-qualified model IDs

## Commands

```bash
gunmetal
gunmetal setup
gunmetal start
gunmetal start --no-open
gunmetal status
gunmetal doctor
gunmetal chat
gunmetal profiles list
gunmetal keys list
gunmetal logs list
```
