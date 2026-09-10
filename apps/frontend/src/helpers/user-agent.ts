export const VISITOR_USER_AGENT_HEADER = 'X-Forwarded-User-Agent'

export function getFrontendUserAgent(commitHash: string): string {
	return `Lumen/frontend/${commitHash || 'unknown'} (support@Lumen.com)`
}
