<template>
	<img
		src="@/assets/brand/octra-mark.png"
		alt=""
		aria-hidden="true"
		draggable="false"
		class="octra-mark"
		:class="{ 'octra-mark--animate': animate }"
	/>
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
.octra-mark {
	display: block;
	object-fit: contain;
	user-select: none;
}

.octra-mark--animate {
	animation: octra-settle 0.9s cubic-bezier(0.32, 0.72, 0, 1) both;
}

@keyframes octra-settle {
	0% {
		opacity: 0;
		transform: scale(0.9) translateY(0.35rem);
	}
	100% {
		opacity: 1;
		transform: scale(1) translateY(0);
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
