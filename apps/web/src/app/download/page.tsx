import type { Metadata } from "next";
import { PackageManagerCommandBox } from "@/components/ui/PackageManagerCommandBox";
import {
  CodeBlock,
  NumberedRow,
  PageFrame,
  PageIntro,
  Panel,
  TextLink,
  releasesUrl,
} from "@/components/ui/MarketingPrimitives";

export const metadata: Metadata = {
  title: "Download",
  description: "Install Gunmetal and start the local API.",
};

export default function DownloadPage() {
  return (
    <PageFrame>
      <div className="grid gap-12 lg:grid-cols-[0.78fr_1.22fr]">
        <PageIntro
          eyebrow="Download"
          title="Install Gunmetal"
          body="Install the CLI globally. The npm package installs the native runtime, then `gunmetal setup` walks the first provider, model sync, and key creation."
        />

        <Panel className="p-5">
          <PackageManagerCommandBox packageName="@dhruv2mars/gunmetal" className="px-0" />
          <div className="mt-6">
            <CodeBlock>{`gunmetal setup
gunmetal web
gunmetal start
gunmetal status`}</CodeBlock>
          </div>
          <div className="mt-6">
            <TextLink href={releasesUrl}>GitHub releases</TextLink>
          </div>
        </Panel>
      </div>

      <section className="mt-20">
        <NumberedRow
          number="01"
          title="Install CLI"
          body="Use the global npm package. It is the public entrypoint for the local Gunmetal runtime."
        />
        <NumberedRow
          number="02"
          title="Run setup"
          body="Setup connects one provider, checks auth, syncs models, creates one key, and prints a ready-to-run test command."
        />
        <NumberedRow
          number="03"
          title="Open Web UI"
          body="Use `gunmetal web` for the local browser UI, or `gunmetal start` for API-only mode."
        />
      </section>
    </PageFrame>
  );
}
