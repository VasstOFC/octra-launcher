type ProjectInfo = { title: string; iconUrl?: string | null };

const cache = new Map<string, ProjectInfo>();

export function modrinthIconUrl(projectId?: string | null, iconUrl?: string | null): string | null {
  if (iconUrl?.startsWith("http")) return iconUrl;
  if (projectId) return `https://cdn.modrinth.com/data/${projectId}/icons/icon.png`;
  return null;
}

export async function fetchModrinthProject(slug: string): Promise<ProjectInfo | null> {
  const key = slug.trim().toLowerCase();
  if (!key) return null;
  const cached = cache.get(key);
  if (cached) return cached;

  try {
    const res = await fetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(slug)}`);
    if (!res.ok) return null;
    const body = (await res.json()) as { title?: string; icon_url?: string };
    const info: ProjectInfo = {
      title: body.title?.trim() || slug,
      iconUrl: body.icon_url || null,
    };
    cache.set(key, info);
    return info;
  } catch {
    return null;
  }
}
