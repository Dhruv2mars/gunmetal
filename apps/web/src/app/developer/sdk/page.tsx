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
  title: "Extension SDK",
  description: "Build native Gunmetal provider integrations.",
};

const sdkRows = [
  {
    number: "01",
    title: "Provider shape",
    body: "Extensions describe auth, model sync, request modes, and provider-specific options behind one local contract.",
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
          title="Extension SDK"
          body="Build powerful native integrations that make a provider feel like part of the local Gunmetal API instead of another remote dashboard."
        />

        <Panel className="p-5">
          <CodeBlock>{`packages/sdk/
packages/sdk-core/
packages/extensions/

provider -> auth status
provider -> sync models
provider -> chat/completions
provider -> responses`}</CodeBlock>
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
