import { defineMessages, type MessageDescriptor } from '@modrinth/ui'

const playtimeMessages = defineMessages({
	hours: {
		id: 'instance.playtime.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	minutes: {
		id: 'instance.playtime.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
	seconds: {
		id: 'instance.playtime.seconds',
		defaultMessage: '{count, plural, one {# second} other {# seconds}}',
	},
})

type FormatMessage = (
	message: MessageDescriptor,
	values?: Record<string, string | number | boolean | Date | null | undefined>,
) => string

export function formatPlaytime(seconds: number, formatMessage: FormatMessage): string | undefined {
	const total = Math.floor(seconds)
	if (total <= 0) {
		return undefined
	}

	const hours = Math.floor(total / 3600)
	if (hours >= 1) {
		return formatMessage(playtimeMessages.hours, { count: hours })
	}

	const minutes = Math.floor(total / 60)
	if (minutes >= 1) {
		return formatMessage(playtimeMessages.minutes, { count: minutes })
	}

	return formatMessage(playtimeMessages.seconds, { count: total })
}
