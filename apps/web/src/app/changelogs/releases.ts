const githubReleasesApiUrl = "https://api.github.com/repos/Dhruv2mars/gunmetal/releases";

type GitHubReleasePayload = {
  body?: string | null;
  draft?: boolean;
  html_url: string;
  name?: string | null;
  prerelease?: boolean;
  published_at: string;
  tag_name: string;
};

export type ReleaseEntry = {
  body: string;
  isPrerelease: boolean;
  publishedAt: string;
  tag: string;
  title: string;
  url: string;
};

export function normalizeGitHubRelease(release: GitHubReleasePayload): ReleaseEntry {
  return {
    body: release.body?.trim() || "Release notes were not provided for this build.",
    isPrerelease: Boolean(release.prerelease),
    publishedAt: release.published_at,
    tag: release.tag_name,
    title: release.name?.trim() || release.tag_name,
    url: release.html_url,
  };
}

export function formatReleaseDate(isoDate: string) {
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(isoDate));
}

export async function fetchGitHubReleases(fetchImpl: typeof fetch = fetch): Promise<ReleaseEntry[]> {
  try {
    const response = await fetchImpl(githubReleasesApiUrl, {
      headers: { Accept: "application/vnd.github+json" },
      next: { revalidate: 3600 },
    });

    if (!response.ok) return [];

    const payload = (await response.json()) as GitHubReleasePayload[];
    if (!Array.isArray(payload)) return [];

    return payload.filter((release) => !release.draft).map(normalizeGitHubRelease);
  } catch {
    return [];
  }
}

export function cleanReleaseBody(body: string) {
  return body
    .replace(/\\n/g, "\n")
    .replace(/\*\*Full Changelog\*\*:[\s\S]*$/i, "")
    .replace(/## What's Changed\n/g, "")
    .replace(/## New Contributors[\s\S]*$/i, "")
    .replace(/^#+\s*/gm, "")
    .replace(/^v?\d+\.\d+\.\d+\s*$/gm, "")
    .replace(/^Verified\s*$/gim, "Verification")
    .replace(/\s*by @[\w-]+\s*/g, " ")
    .replace(/\s*in #\d+/g, "")
    .replace(/\s*\(#\d+\)/g, "")
    .replace(/\[([^\]]+)\]\([^\)]+\)/g, "$1")
    .trim();
}
