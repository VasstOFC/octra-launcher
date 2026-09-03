<template>
	<Transition name="splash-fade" @after-leave="onAfterLeave">
		<div v-if="!doneLoading" class="splash-screen" :class="`${theme.active}-mode`">
			<div class="app-logo-wrapper" data-tauri-drag-region>
				<OctraMark animate class="app-logo text-brand" />
				<p class="app-title">Octra App</p>
				<p class="app-tagline font-minecraft">{{ formatMessage(messages.tagline) }}</p>
				<ProgressBar class="loading-bar" :progress="Math.min(loadingProgress, 100)" />
				<span v-if="message">{{ message }}</span>
			</div>
			<div class="gradient-bg" data-tauri-drag-region></div>
			<div class="cube-bg"></div>
			<div class="base-bg"></div>
		</div>
	</Transition>
</template>

<script setup>
import { defineMessages, injectLoadingState, useVIntl } from '@modrinth/ui'
import { ref, watch } from 'vue'

import OctraMark from '@/components/brand/OctraMark.vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useTheme } from '@/composables/use-theme.ts'

const theme = useTheme()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	tagline: {
		id: 'app.splash.tagline',
		defaultMessage: 'Ready when you are',
	},
})

const doneLoading = ref(false)
const loadingProgress = ref(0)
const message = ref()

const MIN_DISPLAY_MS = 500
const mountedAt = Date.now()

const loading = injectLoadingState()

function onAfterLeave() {
	loading.setEnabled(true)
}

watch(
	[loading.barEnabled, loading.pending],
	([barEnabled, pending]) => {
		if (barEnabled) {
			return
		}

		if (pending) {
			loadingProgress.value = 0
			fakeLoadingIncrease()
			return
		}

		const elapsed = Date.now() - mountedAt
		const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)

		setTimeout(() => {
			if (loading.pending.value) {
				return
			}
			doneLoading.value = true
		}, delay)
	},
	{ immediate: true },
)

function fakeLoadingIncrease() {
	if (loadingProgress.value < 95) {
		setTimeout(() => {
			loadingProgress.value += 2
			fakeLoadingIncrease()
		}, 5)
	}
}

useAppEvent('loading', (e) => {
	if (e.event.type === 'directory_move') {
		loadingProgress.value = 100 * (e.fraction ?? 1)
		message.value = 'Aktualizowanie katalogu aplikacji…'
	}
})
</script>

<style scoped lang="scss">
.splash-screen {
	position: fixed;
	inset: 0;
	z-index: 10000;

	--splash-cube-image: url('@/assets/loading/cube.png');

	&.light-mode {
		--splash-cube-image: url('@/assets/loading/cube-light.webp');
	}
}

.splash-fade-leave-active {
	transition:
		opacity 0.42s cubic-bezier(0.32, 0.72, 0, 1),
		filter 0.42s cubic-bezier(0.32, 0.72, 0, 1);
}

.splash-fade-leave-to {
	opacity: 0;
	filter: blur(4px);
}

@media (prefers-reduced-motion: reduce) {
	.splash-fade-leave-active {
		transition: opacity 0.2s ease;
	}

	.splash-fade-leave-to {
		filter: none;
	}
}

.app-logo-wrapper {
	position: absolute;
	height: 100vh;
	width: 100%;

	display: flex;
	flex-direction: column;
	justify-content: center;
	align-items: center;

	gap: 0.75rem;
	color: var(--color-contrast);

	z-index: 9998;
}

.app-logo {
	height: 5.5rem;
	width: 5.5rem;
}

.app-title {
	margin: 0;
	font-size: 1.75rem;
	font-weight: 800;
	letter-spacing: -0.04em;
	line-height: 1;
	animation: splash-copy-in 0.7s cubic-bezier(0.32, 0.72, 0, 1) 0.12s both;
}

.app-tagline {
	margin: 0;
	font-size: 0.8rem;
	letter-spacing: 0.04em;
	line-height: 1.2;
	color: var(--color-secondary);
	animation: splash-copy-in 0.7s cubic-bezier(0.32, 0.72, 0, 1) 0.22s both;
}

.loading-bar {
	max-width: 20rem;
	margin-top: 0.35rem;
	animation: splash-copy-in 0.7s cubic-bezier(0.32, 0.72, 0, 1) 0.3s both;
}

@keyframes splash-copy-in {
	from {
		opacity: 0;
		transform: translateY(0.4rem);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}

@media (prefers-reduced-motion: reduce) {
	.app-title,
	.app-tagline,
	.loading-bar {
		animation: none;
	}
}

.gradient-bg {
	position: absolute;
	height: 100vh;
	width: 100vw;
	background:
		linear-gradient(180deg, var(--splash-tint-top) 0%, var(--splash-tint-bottom) 97.29%),
		linear-gradient(0deg, var(--splash-overlay), var(--splash-overlay));
	z-index: 9997;
}

.cube-bg {
	position: absolute;

	left: 50%;
	top: 50%;
	transform: translate(-50%, -50%);

	width: 180vw;
	height: 180vh;
	background-color: var(--color-bg);

	z-index: 9996;

	&::after {
		content: '';
		position: absolute;
		inset: 0;
		background: var(--splash-cube-image) center no-repeat;
		background-size: contain;
		opacity: var(--splash-cube-opacity);
		mix-blend-mode: var(--splash-cube-blend);
	}
}

.base-bg {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background: var(--color-bg);
	z-index: 9995;
}
</style>
