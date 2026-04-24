import type { Metadata } from "next";
import {
  PageFrame,
  PageIntro,
  Panel,
  TextLink,
  releasesUrl,
} from "@/components/ui/MarketingPrimitives";
import { cleanReleaseBody, fetchGitHubReleases, formatReleaseDate } from "./releases";

export const metadata: Metadata = {
  title: "Changelogs",
  description: "Gunmetal release notes from GitHub releases.",
};

export const dynamic = "force-dynamic";

function ReleaseNotes({ body }: { body: string }) {
  const lines = cleanReleaseBody(body).split("\n").filter((line) => line.trim());

  return (
    <div className="space-y-2 text-[14px] leading-7 text-[var(--text-muted)]">
      {lines.slice(0, 10).map((line) => {
        const cleaned = line.replace(/^[-*]\s+/, "");
        return (
          <p key={line} className="relative pl-4 before:absolute before:left-0 before:top-[0.8em] before:h-1 before:w-1 before:rounded-full before:bg-[var(--text-faint)]">
            {cleaned}
          </p>
        );
      })}
    </div>
  );
}

export default async function ChangelogsPage() {
  const releases = await fetchGitHubReleases();

  return (
    <PageFrame>
      <PageIntro
        eyebrow="Resources"
        title="Changelogs"
        body="Release notes are pulled from GitHub so this page follows the native runtime, npm wrapper, and web surface without a separate publishing step."
      />

      <section className="mt-16">
        {releases.length > 0 ? (
          <div className="relative">
            <div className="absolute bottom-0 left-[4px] top-2 w-px bg-[rgba(226,226,226,0.10)] md:left-[150px]" />
            {releases.map((release, index) => (
              <article key={release.tag} className="relative grid gap-5 pb-14 last:pb-0 md:grid-cols-[150px_1fr] md:gap-10">
                <div className="hidden pr-8 pt-1 text-right md:block">
                  <p className="text-[13px] text-[var(--text-secondary)]">{formatReleaseDate(release.publishedAt)}</p>
                  <p className="mt-1 font-mono text-[12px] text-[var(--text-faint)]">{release.tag}</p>
                </div>
                <div className="absolute left-0 top-2 h-[9px] w-[9px] rounded-full bg-[var(--text-muted)] ring-4 ring-[var(--bg)] md:left-[146px]" />
                <Panel className="ml-7 p-5 md:ml-0">
                  <div className="flex flex-wrap items-center gap-3">
                    <h2 className="text-[22px] font-normal text-[var(--text)]">{release.title}</h2>
                    {index === 0 ? <span className="font-mono text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">Latest</span> : null}
                    {release.isPrerelease ? <span className="font-mono text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">Prerelease</span> : null}
                  </div>
                  <div className="mt-2 flex gap-3 text-[12px] text-[var(--text-faint)] md:hidden">
                    <span>{release.tag}</span>
                    <span>{formatReleaseDate(release.publishedAt)}</span>
                  </div>
                  <div className="mt-5">
                    <ReleaseNotes body={release.body} />
                  </div>
                  <div className="mt-6">
                    <TextLink href={release.url}>Open release</TextLink>
                  </div>
                </Panel>
              </article>
            ))}
          </div>
        ) : (
          <Panel className="p-8 text-center">
            <h2 className="text-[22px] font-normal text-[var(--text)]">Release feed unavailable</h2>
            <p className="mx-auto mt-3 max-w-[46ch] text-[14px] leading-7 text-[var(--text-muted)]">
              GitHub release notes could not be loaded right now. The release page is still available directly.
            </p>
            <div className="mt-6">
              <TextLink href={releasesUrl}>Go to GitHub</TextLink>
            </div>
          </Panel>
        )}
      </section>
    </PageFrame>
  );
}
