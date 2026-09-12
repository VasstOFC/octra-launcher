import dayjs from 'dayjs'
import { computed, type Ref } from 'vue'

import type { GameInstance } from '@/helpers/types'

export type InstanceTimestamps = {
	lastPlayed: number
	created: number
	modified: number
	hoursPlayed: number
}

const EMPTY_TIMESTAMPS: InstanceTimestamps = {
	lastPlayed: 0,
	created: 0,
	modified: 0,
	hoursPlayed: 0,
}

export function toEpochMs(value: string | number | Date | null | undefined): number {
	if (value === null || value === undefined || value === '') return 0
	if (typeof value === 'number') return value
	if (value instanceof Date) return value.getTime()
	const parsed = dayjs(value)
	return parsed.isValid() ? parsed.valueOf() : 0
}

export function useLibraryTimestamps(instances: Ref<GameInstance[]>) {
	const timestamps = computed(() => {
		const cache = new Map<string, InstanceTimestamps>()
		for (const instance of instances.value) {
			cache.set(instance.id, {
				lastPlayed: toEpochMs(instance.last_played),
				created: toEpochMs(instance.created),
				modified: toEpochMs(instance.modified),
				hoursPlayed: (instance.recent_time_played ?? 0) + (instance.submitted_time_played ?? 0),
			})
		}
		return cache
	})

	const getTimestamps = (instanceId: string): InstanceTimestamps =>
		timestamps.value.get(instanceId) ?? EMPTY_TIMESTAMPS

	return { timestamps, getTimestamps }
}
