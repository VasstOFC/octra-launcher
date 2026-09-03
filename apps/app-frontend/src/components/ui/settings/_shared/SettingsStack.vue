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
	<div class="settings-stack rounded-lg px-3 py-2.5">
		<component
			:is="headingTag"
			:id="labelId"
			class="m-0 text-sm font-medium leading-snug text-contrast"
		>
			{{ title }}
		</component>
		<div class="mt-2 w-full min-w-0 [&>*]:w-full">
			<slot :labelled-by="labelId" :control-id="controlId" />
		</div>
		<p
			v-if="description"
			class="m-0 mt-2 text-sm leading-snug text-secondary"
		>
			{{ description }}
		</p>
	</div>
</template>
