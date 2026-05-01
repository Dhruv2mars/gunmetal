# Gunmetal

Gunmetal turns your AI subscriptions and upstream provider access into a local API. The product language stays small so the installed app, local API, and developer SDK do not blur together.

## Language

**Gunmetal**:
The end-user product that installs a local CLI, daemon/API, and Web UI for routing AI requests through a user's own provider access.
_Avoid_: Gunmetal Suite, suite, platform

**Product promise**:
Gunmetal turns your AI subscriptions and upstream provider access into a local API.
_Avoid_: No API costs, free inference

**Gunmetal Provider SDK**:
The developer-facing product for embedding Gunmetal-style multi-provider routing into software.
_Avoid_: Extension SDK, dashboard SDK, daemon SDK

**Extension**:
A legacy/internal implementation term for provider integration code.
_Avoid_: Public product name

**Provider access**:
A user's existing right to use an upstream AI provider, whether through a paid subscription, browser-authenticated account, free tier, or API key.
_Avoid_: AI subscription, API key, account

**Provider connection**:
One saved connection from Gunmetal to an upstream provider, backed by the user's provider access.
_Avoid_: Profile, account, connection name

**Subscription connection**:
A provider connection backed by browser-authenticated subscription access.
_Avoid_: Free-tier connection

**API-key connection**:
A provider connection backed by an upstream provider API key.
_Avoid_: Free-tier connection

**Upstream provider**:
An external AI service that Gunmetal can route inference requests to.
_Avoid_: Backend, vendor, dashboard

**Local API**:
Gunmetal's local HTTP API that apps call instead of calling upstream providers directly.
_Avoid_: Local API routes, proxy

**Request mode**:
The API shape a local inference request uses, currently `chat/completions` or `responses`.
_Avoid_: Format, endpoint type

**Gunmetal key**:
A local key created by Gunmetal for apps to call the Local API without receiving upstream provider credentials.
_Avoid_: API key, provider key, provider-scoped key, model-scoped key

**App**:
The downstream software where a user wants inference to happen.
_Avoid_: Gunmetal app, provider app

**Local-first**:
Gunmetal stores provider credentials, keys, models, request logs, and dashboard state on the user's machine.
_Avoid_: Hosted dashboard, Gunmetal account, team workspace

**Chat playground**:
The Web UI chat experience where users try models through their Gunmetal keys.
_Avoid_: Single request tester, smoke test, chat

**Dashboard**:
Gunmetal's browser UI for managing provider connections, models, Gunmetal keys, chat, and request history.
_Avoid_: Setup-only admin, local dashboard, hosted dashboard, web app, operator shell

**Provider-qualified model ID**:
A model identifier prefixed by its upstream provider or connection namespace so routing stays explicit.
_Avoid_: Model name, smart route, alias

**Model catalog**:
The Dashboard view of synced provider-qualified models across connected upstream providers.
_Avoid_: Separate model pages per provider

**Explicit routing**:
The rule that users choose the provider-qualified model ID and Gunmetal sends the request through the matching provider connection.
_Avoid_: Cheapest route, fallback route, automatic provider selection

**Credential boundary**:
Upstream credentials stay out of apps and out of any Gunmetal cloud, while Gunmetal uses them to authenticate with the selected upstream provider.
_Avoid_: Credentials never leave the machine

**Request history**:
Gunmetal's local record of requests, errors, latency, token usage, provider, model, and key.
_Avoid_: Cost tracking, budget tracking, spend dashboard

**Codex provider**:
The upstream provider connection for ChatGPT Plus/Pro subscription access through the Codex app server path.
_Avoid_: OpenAI API key, ChatGPT API

**OpenAI provider**:
The upstream provider connection for direct OpenAI API-key access.
_Avoid_: ChatGPT subscription

**Onboarding path**:
Install the CLI, run `gunmetal setup`, then use `gunmetal web` to manage Gunmetal in the local dashboard.
_Avoid_: Web-only setup, hosted signup

## Relationships

- **Gunmetal** is powered by the **Gunmetal Provider SDK**.
- **Gunmetal** is for end users who want local API routes backed by their own provider access.
- **Gunmetal Provider SDK** is for external developers who want to embed multi-provider support.
- **Gunmetal** uses the same **Gunmetal Provider SDK** format that external developers use.
- **Gunmetal Provider SDK** provides routing primitives and provider contracts.
- **Gunmetal** adds the CLI, daemon, **Dashboard**, **Gunmetal keys**, and **Request history** on top of the **Gunmetal Provider SDK**.
- **Extension** may remain in code/package paths until a separate rename, but public language should say **Gunmetal Provider SDK**, **Provider SDK**, or provider integration.
- **Provider access** belongs to an **Upstream provider**.
- A **Provider connection** uses one user's **Provider access** for one **Upstream provider**.
- A user should have at most one **Provider connection** per **Upstream provider**.
- Public UI should not ask users to name a **Provider connection** by default.
- A **Provider connection** is either a **Subscription connection** or an **API-key connection** in public product language.
- Onboarding should present **Subscription connections** before **API-key connections**.
- **Gunmetal** routes local inference requests to **Upstream providers** through the user's **Provider access**.
- The **Product promise** is about unifying existing **Provider access**, not guaranteeing free usage.
- Apps call the **Local API** using a supported **Request mode**.
- **Gunmetal** adapts each **Request mode** to the selected **Upstream provider**.
- Core **Request modes** are chat/responses only for now.
- Apps use a **Gunmetal key** to call the **Local API**.
- A user can create multiple **Gunmetal keys** for the same **Local API**.
- Users can create separate **Gunmetal keys** for different apps.
- An **App** may be a third-party product that accepts custom API keys and base URLs, such as Cursor, Cline, or Open WebUI.
- An **App** may also be software the user built, where they add the **Gunmetal key**, **Local API** base URL, and provider-qualified model ID in code.
- **Gunmetal keys** authorize local API access; requests choose models with **Provider-qualified model IDs**.
- Gunmetal's API model follows the OpenRouter-style pattern: create a **Gunmetal key**, put it in the app, and choose the provider-qualified model in each request.
- Upstream provider credentials stay inside **Gunmetal** and are not given to apps.
- The **Credential boundary** is between apps, Gunmetal, and selected upstream providers.
- **Gunmetal** is **Local-first** and single-user.
- The hosted website is for marketing, docs, and download; it is not the product dashboard.
- The CLI starts onboarding; the **Dashboard** operates **Provider connections**, models, **Gunmetal keys**, the **Chat playground**, and request history.
- Technical docs may call the **Dashboard** the Web UI.
- The **Chat playground** is a real chat experience, not just a one-off request tester.
- The **Chat playground** should call Gunmetal exactly like an external app: **Gunmetal key**, **Local API**, and **Provider-qualified model ID**.
- The **Dashboard** is the daily product surface, not only setup/admin UI.
- Users choose models with **Provider-qualified model IDs**.
- A **Provider-qualified model ID** tells **Gunmetal** which **Provider connection** should handle the request.
- The **Dashboard** should show one **Model catalog** grouped or filtered by **Upstream provider**.
- **Gunmetal** uses **Explicit routing** for requests.
- **Request history** is for inspection and debugging, not cost accounting.
- **Request history** should be a normal **Dashboard** tab, with debugging-first content.
- The **Codex provider** represents ChatGPT Plus/Pro subscription access.
- The **OpenAI provider** represents direct OpenAI API-key access.
- The **Onboarding path** starts in the CLI and continues in the **Dashboard**.

## Example dialogue

> **Dev:** "Should the product page call this Gunmetal Suite?"
> **Domain expert:** "No. The user product is just **Gunmetal**. The separate developer product is **Gunmetal Provider SDK**."
>
> **Dev:** "Is Provider SDK only for building Gunmetal internals?"
> **Domain expert:** "No. **Gunmetal Provider SDK** is external-first, and **Gunmetal** dogfoods the same SDK format."
>
> **Dev:** "Does Provider SDK include the Gunmetal Dashboard?"
> **Domain expert:** "No. **Gunmetal Provider SDK** provides provider contracts and routing primitives. **Gunmetal** adds the user product surfaces."
>
> **Dev:** "Should we rename packages/extensions immediately?"
> **Domain expert:** "No. Update public language first; treat **Extension** as legacy/internal until a separate implementation rename."
>
> **Dev:** "Are we only talking about API keys?"
> **Domain expert:** "No. **Provider access** includes subscriptions like ChatGPT Plus or GitHub Copilot, free provider tiers, and API-key providers."
>
> **Dev:** "Should the UI say provider profile?"
> **Domain expert:** "No. Public copy should say **Provider connection**. Profile can remain an internal implementation term."
>
> **Dev:** "If two users both use GitHub Copilot, is that one provider or two?"
> **Domain expert:** "GitHub Copilot is one **Upstream provider**. Each user has separate **Provider access** and creates their own **Provider connection**."
>
> **Dev:** "Can one user create personal and work OpenRouter connections?"
> **Domain expert:** "No. Keep it simple: one **Provider connection** per **Upstream provider**."
>
> **Dev:** "Should setup ask for a provider profile name?"
> **Domain expert:** "No. Users connect OpenRouter, Codex, or Copilot directly. Connection names are internal unless the product later supports multiple connections per provider."
>
> **Dev:** "Should onboarding show free-tier providers as a category?"
> **Domain expert:** "No. Public categories are **Subscription connection** and **API-key connection** only."
>
> **Dev:** "Which connection type comes first?"
> **Domain expert:** "**Subscription connections** come first because they are the distinctive Gunmetal use case."
>
> **Dev:** "Can we say Gunmetal removes API costs?"
> **Domain expert:** "No. Say it lets users use the AI access they already have through one local API."
>
> **Dev:** "Is the local API only OpenAI-compatible?"
> **Domain expert:** "No. Publicly call it an OpenAI-compatible **Local API**, but model the supported **Request modes** explicitly because providers use different upstream shapes."
>
> **Dev:** "Should Gunmetal's core story include embeddings, images, speech, or reranking?"
> **Domain expert:** "No. Keep the core story to chat/responses for now."
>
> **Dev:** "Should users paste upstream provider keys into every app?"
> **Domain expert:** "No. Apps should receive a **Gunmetal key**. Upstream provider credentials stay inside **Gunmetal**."
>
> **Dev:** "What is an app in Gunmetal copy?"
> **Domain expert:** "An **App** is the downstream inference client: a third-party product that accepts API keys/base URLs, or software the user controls and can wire like an OpenRouter-style API."
>
> **Dev:** "Why can a user create multiple Gunmetal keys?"
> **Domain expert:** "So each app can have its own **Gunmetal key** while sharing the same **Local API**."
>
> **Dev:** "Should users choose providers or models when creating a Gunmetal key?"
> **Domain expert:** "No. Create the **Gunmetal key** first. The app chooses provider-qualified models in its request code."
>
> **Dev:** "Where does model selection happen?"
> **Domain expert:** "In the request payload, using a **Provider-qualified model ID**, similar to OpenRouter-style model selection."
>
> **Dev:** "Should each provider connection have its own model page?"
> **Domain expert:** "No. The **Dashboard** should show one **Model catalog** with provider grouping/filtering."
>
> **Dev:** "Can we say upstream credentials never leave the machine?"
> **Domain expert:** "No. Say they stay out of apps and Gunmetal cloud, and Gunmetal uses them only to authenticate with the selected **Upstream provider**."
>
> **Dev:** "Should Gunmetal promise spend tracking?"
> **Domain expert:** "No. **Request history** tracks requests and usage details for inspection, not billing or budgets."
>
> **Dev:** "Is Request history hidden developer logging?"
> **Domain expert:** "No. **Request history** is a normal **Dashboard** tab, but its content stays debugging-first rather than analytics-first."
>
> **Dev:** "Why is ChatGPT subscription access called Codex?"
> **Domain expert:** "Because ChatGPT Plus/Pro subscription inference is reached through the Codex app server path, while **OpenAI provider** means direct API-key access."
>
> **Dev:** "Should Gunmetal have teams or a hosted dashboard?"
> **Domain expert:** "No. **Gunmetal** is single-user and **Local-first**. The dashboard runs on the user's machine."
>
> **Dev:** "Is the playground only for one smoke-test request?"
> **Domain expert:** "No. The **Chat playground** should feel like an actual chat app using the user's **Gunmetal keys**."
>
> **Dev:** "Can the Dashboard chat bypass Gunmetal keys internally?"
> **Domain expert:** "No. The **Chat playground** should use the same **Gunmetal key** and **Local API** path an external app uses."
>
> **Dev:** "Should public copy call it the Web UI or local dashboard?"
> **Domain expert:** "Call it the **Dashboard** everywhere public. Use Web UI only in technical details."
>
> **Dev:** "Is the Dashboard only for setup?"
> **Domain expert:** "No. The **Dashboard** is where users manage routing and chat with models day to day."
>
> **Dev:** "Should Gunmetal hide routing behind smart aliases?"
> **Domain expert:** "No. Keep model routing explicit with **Provider-qualified model IDs**, matching the current implementation."
>
> **Dev:** "Should Gunmetal automatically choose the cheapest provider?"
> **Domain expert:** "No. **Gunmetal** uses **Explicit routing** for now."
>
> **Dev:** "Should first-run setup be CLI-first or Web UI-first?"
> **Domain expert:** "The CLI starts setup, then `gunmetal web` opens the local dashboard for ongoing management."

## Flagged ambiguities

- "Gunmetal Suite" was used as a public product name, but the resolved product name is **Gunmetal**.
- "Extension SDK" was used as a public product name, but the resolved developer product name is **Gunmetal Provider SDK**.
- "Extension" can remain as an internal implementation term, but should not be a public product noun.
- "subscription", "account", and "API key" each describe only part of **Provider access**.
- Free-tier access explains a use case, but should not appear as a public provider category.
- "profile" exists in code, but public product language should use **Provider connection**.
- Hosted website pages should not imply a hosted product dashboard, team account, or cloud control plane.
- ChatGPT Plus/Pro subscription access should not be merged with direct OpenAI API-key access; use **Codex provider** and **OpenAI provider** for those separate paths.
