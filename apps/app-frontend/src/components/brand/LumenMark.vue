<template>
	<img
		src="@/assets/brand/lumen-mark.png"
		alt=""
		aria-hidden="true"
		draggable="false"
		class="lumen-mark"
		:class="{ 'lumen-mark--animate': animate }"
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
.lumen-mark {
	display: block;
	object-fit: contain;
	user-select: none;
}

.lumen-mark--animate {
	animation: Lumen-settle 0.9s cubic-bezier(0.32, 0.72, 0, 1) both;
}

@keyframes lumen-settle {
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
	.lumen-mark--animate {
		animation: none;
		opacity: 1;
		transform: none;
	}
}
</style>
