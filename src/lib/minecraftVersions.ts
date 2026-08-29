import type { ManifestVersion } from "../types";

export type VersionBucket = "current" | "snapshot" | "legacy";

export function isLegacyMinecraftVersion(v: ManifestVersion): boolean {
  if (v.versionType === "old_alpha" || v.versionType === "old_beta") return true;

  const release = v.id.match(/^(\d+)\.(\d+)/);
  if (release) {
    const major = Number(release[1]);
    const minor = Number(release[2]);
    return major < 1 || (major === 1 && minor <= 18);
  }

  if (v.versionType === "snapshot") {
    const snap = v.id.match(/^(\d{2})w/);
    if (snap) return Number(snap[1]) <= 21;
    return true;
  }

  return v.versionType !== "release";
}

export function versionBucket(v: ManifestVersion): VersionBucket {
  if (isLegacyMinecraftVersion(v)) return "legacy";
  if (v.versionType === "snapshot") return "snapshot";
  return "current";
}

export function versionTypeLabel(type: string): string {
  switch (type) {
    case "release":
      return "Release";
    case "snapshot":
      return "Snapshot";
    case "old_beta":
      return "Beta";
    case "old_alpha":
      return "Alpha";
    default:
      return type;
  }
}

export function bucketVersions(versions: ManifestVersion[]) {
  const current: ManifestVersion[] = [];
  const snapshot: ManifestVersion[] = [];
  const legacy: ManifestVersion[] = [];

  for (const v of versions) {
    const bucket = versionBucket(v);
    if (bucket === "legacy") legacy.push(v);
    else if (bucket === "snapshot") snapshot.push(v);
    else current.push(v);
  }

  return { current, snapshot, legacy };
}

export function filterVersions(
  versions: ManifestVersion[],
  query: string,
  bucket: VersionBucket | "all",
): ManifestVersion[] {
  const q = query.trim().toLowerCase();
  return versions.filter((v) => {
    if (bucket !== "all" && versionBucket(v) !== bucket) return false;
    if (!q) return true;
    return (
      v.id.toLowerCase().includes(q) ||
      v.versionType.toLowerCase().includes(q) ||
      versionTypeLabel(v.versionType).toLowerCase().includes(q)
    );
  });
}
