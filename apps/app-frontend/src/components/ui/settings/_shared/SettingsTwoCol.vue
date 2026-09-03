<script setup lang="ts">
import { useSlots } from 'vue'

withDefaults(
	defineProps<{
		gap?: 'sm' | 'md' | 'lg'
	}>(),
	{
		gap: 'md',
	},
)

const slots = useSlots()
</script>

<template>
	<div
		class="settings-two-col grid grid-cols-1 md:grid-cols-2"
		:class="{
			'gap-3': gap === 'sm',
			'gap-4': gap === 'md',
			'gap-6': gap === 'lg',
		}"
	>
		<template v-if="slots.left || slots.right">
			<div class="min-w-0">
				<slot name="left" />
			</div>
			<div class="min-w-0">
				<slot name="right" />
			</div>
		</template>
		<slot v-else />
	</div>
</template>
