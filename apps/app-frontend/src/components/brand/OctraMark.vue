<template>
	<svg
		viewBox="0 0 64 64"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
		aria-hidden="true"
		:class="{ 'octra-mark--animate': animate }"
	>
		<rect width="64" height="64" rx="14" fill="#12141a" stroke="currentColor" stroke-width="2" />
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
.octra-mark--animate {
	color: var(--color-brand);
	animation: octra-settle 0.9s cubic-bezier(0.32, 0.72, 0, 1) both;
}

@media (prefers-reduced-motion: no-preference) {
	.octra-mark--animate {
		animation:
			octra-settle 0.9s cubic-bezier(0.32, 0.72, 0, 1) both,
			octra-glow 2.8s ease-in-out 0.9s infinite;
	}
}

@keyframes octra-settle {
	0% {
		opacity: 0;
		transform: scale(0.86) translateY(0.5rem);
		filter: drop-shadow(0 0 0 transparent);
	}
	100% {
		opacity: 1;
		transform: scale(1) translateY(0);
		filter: drop-shadow(0 0 0.45rem color-mix(in srgb, var(--color-brand) 45%, transparent));
	}
}

@keyframes octra-glow {
	0%,
	100% {
		filter: drop-shadow(0 0 0.35rem color-mix(in srgb, var(--color-brand) 35%, transparent));
	}
	50% {
		filter: drop-shadow(0 0 0.7rem color-mix(in srgb, var(--color-brand) 65%, transparent));
	}
}

@media (prefers-reduced-motion: reduce) {
	.octra-mark--animate {
		animation: none;
		opacity: 1;
		transform: none;
	}
}
</style>
