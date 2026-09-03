<script setup lang="ts">
defineProps<{
	title: string
	description?: string
}>()
</script>

<template>
	<header class="settings-panel-header mb-6">
		<h2 class="m-0 text-xl font-semibold leading-tight text-contrast">
			{{ title }}
		</h2>
		<p
			v-if="description"
			class="m-0 mt-1.5 max-w-2xl text-sm leading-snug text-secondary"
		>
			{{ description }}
		</p>
		<slot />
	</header>
</template>
