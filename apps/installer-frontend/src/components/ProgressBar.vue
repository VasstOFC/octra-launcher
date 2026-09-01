<template>
	<div class="progress-bar">
		<div class="progress-bar__fill" :style="{ width: `${progress}%` }" />
	</div>
</template>

<script setup lang="ts">
defineProps<{ progress: number }>()
</script>

<style scoped>
.progress-bar {
	width: 100%;
	height: 0.5rem;
	background-color: var(--color-button-bg);
	border-radius: var(--radius-lg);
	overflow: hidden;
}

.progress-bar__fill {
	height: 100%;
	background: linear-gradient(to right, var(--color-brand), #e0b3ff);
	transition: width 0.25s ease-out;
}
</style>
