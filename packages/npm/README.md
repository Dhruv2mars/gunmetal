# @dhruv2mars/gunmetal

Install Gunmetal and run the local AI switchboard.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

Install downloads the native `gunmetal` binary into `~/.gunmetal/bin/`.

## Quickstart

```bash
gunmetal setup
gunmetal start
gunmetal status
```

`gunmetal setup` is the default path. It saves one profile, checks auth, syncs models, creates one key, and prints the next test command.

## Start Here

```bash
export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key

curl $OPENAI_BASE_URL/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

curl $OPENAI_BASE_URL/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-5.1",
    "messages": [{"role":"user","content":"say ok"}]
  }'
```

Then point your app at:

- base URL: `http://127.0.0.1:4684/v1`
- API key: the Gunmetal key created during setup
- model: a provider/model id like `codex/gpt-5.4`

Gunmetal works when the app talks to Gunmetal:

- custom base URL
- custom API key
- arbitrary model names
- if the app hardcodes the upstream endpoint, Gunmetal cannot help there

## Commands

```bash
gunmetal
gunmetal setup
gunmetal start
gunmetal status
gunmetal profiles list
gunmetal keys list
gunmetal logs list
```
