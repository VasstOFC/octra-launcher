<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		title: string
		description?: string
		controlId?: string
		headingLevel?: 2 | 3 | 4
	}>(),
	{
		headingLevel: 3,
	},
)

const headingTag = computed(() => `h${props.headingLevel}`)
const labelId = computed(() =>
	props.controlId ? `${props.controlId}-label` : undefined,
)
</script>

<template>
	<div
		class="settings-row flex min-h-12 items-start justify-between gap-4 rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-3 sm:items-center"
	>
		<div class="min-w-0 flex-1 overflow-hidden">
			<component
				:is="headingTag"
				:id="labelId"
				class="m-0 text-sm font-medium leading-snug text-contrast"
			>
				{{ title }}
			</component>
			<p
				v-if="description"
				class="m-0 mt-0.5 text-sm leading-snug text-secondary"
			>
				{{ description }}
			</p>
		</div>
		<div class="flex max-w-[min(100%,16rem)] shrink-0 items-center justify-end">
			<slot :labelled-by="labelId" :control-id="controlId" />
		</div>
	</div>
</template>
