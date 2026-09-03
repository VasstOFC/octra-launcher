<script setup lang="ts">
import { Avatar, truncatedTooltip } from '@modrinth/ui'
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
</script>

<template>
	<div
		class="instance-card-view relative flex w-full min-w-0 select-none overflow-clip text-left transition-colors"
		:class="{
			'flex-row items-center justify-start gap-3 rounded-lg px-2.5 py-2': compactMode,
			'flex-col items-start justify-end gap-2.5 rounded-xl bg-surface-2 p-2.5': !compactMode,
			'bg-surface-3': selected,
			'bg-transparent hover:bg-surface-3': compactMode && !selected,
			'hover:bg-surface-3': !compactMode && !selected,
			'ring-1 ring-inset ring-[color-mix(in_srgb,var(--color-contrast)_30%,transparent)]':
				selected,
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
			<slot name="loading" :compact="compactMode" />
			<div
				v-if="!compactMode"
				class="absolute bottom-1.5 right-1.5 z-[1] flex size-11 items-center justify-center"
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
		</div>
		<div v-if="compactMode" class="relative flex shrink-0 items-center justify-center">
			<slot name="leading" :compact="compactMode" />
		</div>
		<slot name="overlay" :compact="compactMode" />
	</div>
</template>
