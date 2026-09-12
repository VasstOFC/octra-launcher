<script setup lang="ts">
import { Avatar, truncatedTooltip } from '@lumen/ui'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

dayjs.extend(relativeTime)

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		selected?: boolean
	}>(),
	{
		selected: false,
	},
)

const iconSrc = computed(() => getInstanceIconUrl(props.instance.icon_path))
const appSettings = useAppSettings()
const compactMode = computed(() => appSettings.getFeatureFlag('compact_instance_cards'))

const nameRef = ref<HTMLElement | null>(null)
const versionRef = ref<HTMLElement | null>(null)

const lastPlayedText = computed(() => {
	if (!props.instance.last_played) return null
	return dayjs(props.instance.last_played).fromNow()
})

const playtimeText = computed(() => {
	if (!props.instance.playtime || props.instance.playtime === 0) return null
	const hours = Math.floor(props.instance.playtime / 3600)
	const minutes = Math.floor((props.instance.playtime % 3600) / 60)
	if (hours > 0) return `${hours}h ${minutes}m`
	return `${minutes}m`
})

const metaText = computed(() => {
	const parts = [`${props.instance.loader} ${props.instance.game_version}`]
	if (lastPlayedText.value) parts.push(lastPlayedText.value)
	if (playtimeText.value) parts.push(playtimeText.value)
	return parts.join(' · ')
})
</script>

<template>
	<div
		class="instance-card-view relative flex w-full min-w-0 select-none overflow-clip text-left transition-all duration-200"
		:class="{
			'flex-row items-center justify-start gap-3 rounded-lg px-2.5 py-2': compactMode,
			'flex-row items-center gap-3 rounded-xl p-3': !compactMode,
			'instance-card--selected': selected,
			'instance-card--compact-hover': compactMode && !selected,
			'instance-card--card-hover': !compactMode && !selected,
			'ring-1 ring-inset ring-[color-mix(in_srgb,var(--color-contrast)_30%,transparent)]': selected,
		}"
	>
		<div
			class="relative flex shrink-0 items-center overflow-clip"
			:class="compactMode ? 'size-11 rounded-lg' : 'size-14 rounded-[10px]'"
		>
			<Avatar
				class="pointer-events-none outline-none !rounded-lg"
				size="100%"
				:src="iconSrc"
				:tint-by="instance.id"
				alt=""
				no-shadow
				pad-transparent-corners
			/>
			<slot name="loading" :compact="compactMode" />
		</div>
		<div class="flex min-w-0 flex-1 flex-col items-start justify-center gap-0.5">
			<p
				ref="nameRef"
				v-tooltip="truncatedTooltip(nameRef, instance.name)"
				class="m-0 w-full truncate text-base font-semibold leading-5 text-contrast"
			>
				{{ instance.name }}
			</p>
			<p
				ref="versionRef"
				v-tooltip="truncatedTooltip(versionRef, metaText)"
				class="m-0 w-full truncate text-sm font-medium capitalize leading-[18px] text-secondary"
			>
				{{ metaText }}
			</p>
		</div>
		<div class="relative flex shrink-0 items-center justify-center">
			<slot name="leading" :compact="compactMode" />
		</div>
		<slot name="overlay" :compact="compactMode" />
	</div>
</template>

<style scoped>
.instance-card--selected {
	background: rgba(0, 212, 255, 0.08) !important;
	border: 1px solid rgba(0, 212, 255, 0.2);
	box-shadow: 0 0 12px rgba(0, 212, 255, 0.1);
}

.instance-card--compact-hover {
	transition: all 0.2s ease;
}

.instance-card--compact-hover:hover {
	background: rgba(0, 212, 255, 0.06) !important;
	box-shadow: 0 0 8px rgba(0, 212, 255, 0.08);
}

.instance-card--card-hover {
	background: rgba(20, 20, 35, 0.7) !important;
	backdrop-filter: blur(12px);
	-webkit-backdrop-filter: blur(12px);
	border: 1px solid rgba(255, 255, 255, 0.06);
	transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.instance-card--card-hover:hover {
	background: rgba(25, 25, 45, 0.85) !important;
	border-color: rgba(0, 212, 255, 0.25);
	box-shadow:
		0 0 0 1px rgba(0, 212, 255, 0.12),
		0 0 20px -5px rgba(0, 212, 255, 0.25);
}
</style>
