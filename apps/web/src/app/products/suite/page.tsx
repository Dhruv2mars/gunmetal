import type { Metadata } from "next";
import {
  CodeBlock,
  NumberedRow,
  PageFrame,
  PageIntro,
  Panel,
  TextLink,
  localApiUrl,
  localAppUrl,
} from "@/components/ui/MarketingPrimitives";

export const metadata: Metadata = {
  title: "Gunmetal Suite",
  description: "The local-first Gunmetal workflow: providers, models, keys, and request history.",
};

const suiteRows = [
  {
    number: "01",
    title: "Providers",
    body: "Save one upstream provider profile. Browser-session providers open auth. API-key providers keep credentials on this machine.",
  },
  {
    number: "02",
    title: "Models",
    body: "Sync provider models into one local registry so apps can target explicit ids like codex/gpt-5.4 or openai/gpt-5.1.",
  },
  {
    number: "03",
    title: "Keys",
    body: "Mint a Gunmetal key and give that to apps. Upstream provider keys stay behind the local daemon.",
  },
  {
    number: "04",
    title: "Requests",
    body: "Inspect success, failure, latency, token use, key, provider, model, and endpoint from local request history.",
  },
];

export default function ProductSuitePage() {
  return (
    <PageFrame>
      <div className="grid gap-12 lg:grid-cols-[0.78fr_1.22fr]">
        <PageIntro
          eyebrow="Products"
          title="Gunmetal Suite"
          body="The all-in-one platform for your needs: connect providers, sync models, mint local keys, and watch traffic through one OpenAI-compatible local API."
        />

        <Panel className="p-5">
          <CodeBlock>{`app/tool
  -> Gunmetal key
  -> ${localApiUrl}
  -> provider extension
  -> upstream provider`}</CodeBlock>
          <div className="mt-6 grid gap-3 sm:grid-cols-2">
            <div className="rounded-lg border border-[rgba(226,226,226,0.08)] p-4">
              <p className="font-mono text-[12px] text-[var(--text-faint)]">Browser UI</p>
              <p className="mt-2 font-mono text-[13px] text-[var(--text-secondary)]">{localAppUrl}</p>
            </div>
            <div className="rounded-lg border border-[rgba(226,226,226,0.08)] p-4">
              <p className="font-mono text-[12px] text-[var(--text-faint)]">Local API</p>
              <p className="mt-2 font-mono text-[13px] text-[var(--text-secondary)]">{localApiUrl}</p>
            </div>
          </div>
          <div className="mt-6">
            <TextLink href="/download">Download Gunmetal</TextLink>
          </div>
        </Panel>
      </div>

      <section className="mt-20">
        {suiteRows.map((row) => (
          <NumberedRow key={row.number} {...row} />
        ))}
      </section>
    </PageFrame>
  );
}
