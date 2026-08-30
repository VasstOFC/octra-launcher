<template>
	<svg
		viewBox="0 0 64 64"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
		aria-hidden="true"
		:class="{ 'octra-mark--spin': animate }"
	>
		<rect width="64" height="64" rx="14" fill="#120c14" stroke="currentColor" stroke-width="2" />
		<circle cx="32" cy="32" r="14" fill="none" stroke="currentColor" stroke-width="6" />
	</svg>
</template>

<script setup lang="ts">
defineProps({
	animate: {
		type: Boolean,
		default: false,
	},
})
</script>

<style scoped>
.octra-mark--spin {
	color: var(--color-brand);
	animation: octra-pulse 2.4s ease-in-out infinite;
}

@keyframes octra-pulse {
	0%,
	100% {
		filter: drop-shadow(0 0 0.25rem color-mix(in srgb, var(--color-brand) 35%, transparent));
	}
	50% {
		filter: drop-shadow(0 0 0.85rem color-mix(in srgb, var(--color-brand) 80%, transparent));
	}
}
</style>
