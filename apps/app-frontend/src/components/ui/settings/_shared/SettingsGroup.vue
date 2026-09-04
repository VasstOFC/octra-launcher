<script setup lang="ts">
defineProps<{
	label: string
	description?: string
}>()
</script>

<template>
	<section
		class="settings-group border-0 border-solid border-surface-5 pt-4 first-of-type:pt-0 [&:not(:first-of-type)]:mt-4 [&:not(:first-of-type)]:border-t"
	>
		<div class="mb-3">
			<h3
				class="m-0 text-xs font-medium uppercase tracking-wide text-secondary"
			>
				{{ label }}
			</h3>
			<p
				v-if="description"
				class="m-0 mt-1 text-sm leading-snug text-secondary"
			>
				{{ description }}
			</p>
		</div>
		<div class="flex flex-col gap-0.5">
			<slot />
		</div>
	</section>
</template>
