import type { Metadata } from "next";
import {
  CodeBlock,
  NumberedRow,
  PageFrame,
  PageIntro,
  Panel,
  TextLink,
  repoUrl,
} from "@/components/ui/MarketingPrimitives";

export const metadata: Metadata = {
  title: "Gunmetal Provider SDK",
  description: "Build native Gunmetal provider contracts and routing primitives.",
};

const sdkRows = [
  {
    number: "01",
    title: "Provider contracts",
    body: "Provider SDK integrations describe auth, model sync, request modes, and provider-specific options behind one local contract.",
  },
  {
    number: "02",
    title: "Local trust",
    body: "Provider credentials and browser sessions remain local. Gunmetal keys are the only keys apps need.",
  },
  {
    number: "03",
    title: "Normalized first",
    body: "Use normalized OpenAI-compatible requests by default. Use passthrough only when provider-native behavior is needed.",
  },
];

export default function DeveloperSdkPage() {
  return (
    <PageFrame>
      <div className="grid gap-12 lg:grid-cols-[0.78fr_1.22fr]">
        <PageIntro
          eyebrow="Developer"
          title="Gunmetal Provider SDK"
          body="Build against the provider contracts and routing primitives Gunmetal dogfoods internally, without the Dashboard, daemon, key-management UI, or request history product surfaces."
        />

        <Panel className="p-5">
          <CodeBlock>{`cargo add gunmetal-sdk gunmetal-core gunmetal-storage
cargo add gunmetal-providers

provider -> auth status
provider -> sync models
provider -> chat/completions
provider -> responses

ProviderHub::new(paths, registry)
builtin_provider_hub(paths)`}</CodeBlock>
          <div className="mt-6 flex flex-wrap gap-4">
            <TextLink href={`${repoUrl}/tree/main/packages/sdk`}>SDK package</TextLink>
            <TextLink href={`${repoUrl}/tree/main/packages/extensions`}>Extensions</TextLink>
          </div>
        </Panel>
      </div>

      <section className="mt-20">
        {sdkRows.map((row) => (
          <NumberedRow key={row.number} {...row} />
        ))}
      </section>
    </PageFrame>
  );
}
