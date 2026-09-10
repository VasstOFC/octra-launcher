<script setup lang="ts">
import { Avatar, truncatedTooltip } from '@lumen/ui'
import dayjs from 'dayjs'
import { computed, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

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
</script>

<template>
	<div
		class="instance-card-view relative flex w-full min-w-0 select-none overflow-clip text-left transition-all duration-200"
		:class="{
			'flex-row items-center justify-start gap-3 rounded-lg px-2.5 py-2': compactMode,
			'flex-col items-start justify-end gap-2.5 rounded-xl p-3': !compactMode,
			'instance-card--selected': selected,
			'instance-card--compact-hover': compactMode && !selected,
			'instance-card--card-hover': !compactMode && !selected,
			'ring-1 ring-inset ring-[color-mix(in_srgb,var(--color-contrast)_30%,transparent)]': selected,
		}"
	>
		<div
			class="relative flex shrink-0 items-center overflow-clip"
			:class="compactMode ? 'size-11 rounded-lg' : 'aspect-square min-w-full rounded-lg'"
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
			<div v-if="!compactMode" class="absolute inset-0 card-overlay" />
			<slot name="loading" :compact="compactMode" />
			<div
				v-if="!compactMode"
				class="absolute bottom-2 right-2 z-[1] flex size-11 items-center justify-center"
			>
				<slot name="leading" :compact="compactMode" />
			</div>
		</div>
		<div class="flex min-w-0 flex-1 flex-col items-start justify-center gap-0.5 px-0.5">
			<p
				ref="nameRef"
				v-tooltip="truncatedTooltip(nameRef, instance.name)"
				class="m-0 w-full truncate text-base font-semibold leading-5 text-contrast"
			>
				{{ instance.name }}
			</p>
			<p
				ref="versionRef"
				v-tooltip="truncatedTooltip(versionRef, `${instance.loader} ${instance.game_version}`)"
				class="m-0 w-full truncate text-sm font-medium capitalize leading-[18px] text-primary"
			>
				{{ instance.loader }} {{ instance.game_version }}
			</p>
			<div v-if="!compactMode && (lastPlayedText || playtimeText)" class="mt-1 flex items-center gap-2 text-xs text-secondary">
				<span v-if="lastPlayedText" class="flex items-center gap-1">
					<svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="10" />
						<polyline points="12 6 12 12 16 14" />
					</svg>
					{{ lastPlayedText }}
				</span>
				<span v-if="playtimeText" class="flex items-center gap-1">
					<svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
					</svg>
					{{ playtimeText }}
				</span>
			</div>
		</div>
		<div v-if="compactMode" class="relative flex shrink-0 items-center justify-center">
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
	border-color: rgba(0, 212, 255, 0.15);
	box-shadow:
		0 0 0 1px rgba(0, 212, 255, 0.08),
		0 8px 25px -5px rgba(0, 0, 0, 0.4);
	transform: translateY(-3px);
}

.card-overlay {
	background: linear-gradient(
		to top,
		rgba(10, 10, 15, 0.8) 0%,
		rgba(10, 10, 15, 0.2) 40%,
		transparent 100%
	);
	pointer-events: none;
}
</style>
