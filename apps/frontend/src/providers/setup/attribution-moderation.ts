import { attributionQuickReplies } from '@lumen/moderation'
import { provideAttributionModeration } from '@lumen/ui'

export function setupAttributionModerationProvider() {
	provideAttributionModeration({ attributionQuickReplies })
}
