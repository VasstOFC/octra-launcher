import type { CrowdinMessages } from '@lumen/ui'

export const moderationLocaleModules = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)
