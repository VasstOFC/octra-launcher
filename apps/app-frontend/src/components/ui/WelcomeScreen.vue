<script setup lang="ts">
import { ImportIcon, PlusIcon } from '@lumen/assets'
import { Button, defineMessages, IntlFormatted, useVIntl } from '@lumen/ui'
import { inject, onMounted, onUnmounted, ref } from 'vue'

import LumenMark from '@/components/brand/LumenMark.vue'
import FeaturedPackCard from '@/components/ui/FeaturedPackCard.vue'

const showCreationModal = inject<() => void>('showCreationModal')
const showImportModal = inject<() => void>('showImportModal')

const { formatMessage } = useVIntl()

const messages = defineMessages({
	welcomeTitle: {
		id: 'app.welcome-screen.title',
		defaultMessage: 'Welcome to Lumen App',
	},
	welcomeDescription: {
		id: 'app.welcome-screen.description',
		defaultMessage: 'Ready to start playing?',
	},
	createInstance: {
		id: 'app.welcome-screen.create-instance',
		defaultMessage: 'Create an instance',
	},
	quickCreateHint: {
		id: 'app.welcome-screen.quick-create-hint',
		defaultMessage: 'Press <shortcut>N</shortcut> to quick create an instance',
	},
	importPrompt: {
		id: 'app.welcome-screen.import-prompt',
		defaultMessage: 'Escaping another launcher?',
	},
	importFromLauncher: {
		id: 'app.welcome-screen.import-from-launcher',
		defaultMessage: 'Import from launcher',
	},
})

const offline = ref(!navigator.onLine)

function handleOffline() {
	offline.value = true
}

function handleOnline() {
	offline.value = false
}

function handleQuickCreate(event: KeyboardEvent) {
	const target = event.target as HTMLElement | null
	if (
		event.key.toLowerCase() !== 'n' ||
		event.repeat ||
		event.metaKey ||
		event.ctrlKey ||
		event.altKey ||
		target?.isContentEditable ||
		['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName ?? '')
	) {
		return
	}

	if (!offline.value) {
		event.preventDefault()
		showCreationModal?.()
	}
}

function particleStyle(i: number) {
	const size = 2 + Math.random() * 4
	const x = Math.random() * 100
	const y = Math.random() * 100
	const delay = Math.random() * 10
	const duration = 8 + Math.random() * 15
	const opacity = 0.1 + Math.random() * 0.4
	return {
		width: `${size}px`,
		height: `${size}px`,
		left: `${x}%`,
		top: `${y}%`,
		animationDelay: `${delay}s`,
		animationDuration: `${duration}s`,
		opacity,
	}
}

onMounted(() => {
	window.addEventListener('offline', handleOffline)
	window.addEventListener('online', handleOnline)
	window.addEventListener('keydown', handleQuickCreate)
})

onUnmounted(() => {
	window.removeEventListener('offline', handleOffline)
	window.removeEventListener('online', handleOnline)
	window.removeEventListener('keydown', handleQuickCreate)
})
</script>

<template>
	<div class="welcome-screen">
		<div class="welcome-bg" aria-hidden="true">
			<div class="gradient-mesh" />
			<div class="particles">
				<span v-for="i in 30" :key="i" class="particle" :style="particleStyle(i)" />
			</div>
			<div class="grid-pattern" />
		</div>
		<div class="relative flex grow items-center justify-center">
			<div class="welcome-content">
				<div class="logo-wrapper">
					<div class="logo-glow">
						<LumenMark class="pointer-events-none size-full" />
					</div>
				</div>
				<div class="text-center">
					<h1 class="welcome-title">
						{{ formatMessage(messages.welcomeTitle) }}
					</h1>
					<p class="welcome-subtitle font-minecraft">
						{{ formatMessage(messages.welcomeDescription) }}
					</p>
				</div>
				<div class="welcome-actions">
					<div class="pack-card-wrapper">
						<FeaturedPackCard compact />
					</div>
					<Button
						type="colored"
						color="brand"
						size="lg"
						class="welcome-cta"
						:disabled="offline"
						@click="showCreationModal?.()"
					>
						<PlusIcon />
						{{ formatMessage(messages.createInstance) }}
					</Button>
					<span class="hint-text">
						<IntlFormatted :message-id="messages.quickCreateHint">
							<template #shortcut="{ children }">
								<kbd class="shortcut-key">
									<component :is="() => children" />
								</kbd>
							</template>
						</IntlFormatted>
					</span>
				</div>
			</div>
		</div>
		<div class="welcome-footer">
			<span class="whitespace-nowrap">{{ formatMessage(messages.importPrompt) }}</span>
			<Button size="lg" class="import-btn" :disabled="offline" @click="showImportModal?.()">
				<ImportIcon />
				{{ formatMessage(messages.importFromLauncher) }}
			</Button>
		</div>
	</div>
</template>

<style scoped>
.welcome-screen {
	position: relative;
	display: flex;
	flex-direction: column;
	min-height: 100%;
	overflow: hidden;
}

.welcome-bg {
	position: absolute;
	inset: 0;
	pointer-events: none;
	overflow: hidden;
}

.gradient-mesh {
	position: absolute;
	inset: 0;
	background:
		radial-gradient(ellipse at 20% 30%, rgba(31, 107, 79, 0.12) 0%, transparent 50%),
		radial-gradient(ellipse at 80% 70%, rgba(31, 107, 79, 0.07) 0%, transparent 50%),
		radial-gradient(ellipse at 50% 50%, rgba(31, 107, 79, 0.04) 0%, transparent 70%);
}

.particles {
	position: absolute;
	inset: 0;
}

.particle {
	position: absolute;
	background: rgba(85, 169, 130, 0.55);
	border-radius: 50%;
	animation: particle-float ease-in-out infinite;
	filter: blur(1px);
}

@keyframes particle-float {
	0%, 100% {
		transform: translateY(0) translateX(0);
		opacity: 0;
	}
	10% {
		opacity: var(--particle-opacity, 0.3);
	}
	50% {
		transform: translateY(-30px) translateX(15px);
	}
	90% {
		opacity: var(--particle-opacity, 0.3);
	}
}

.grid-pattern {
	position: absolute;
	inset: 0;
	background-image:
		linear-gradient(rgba(255, 255, 255, 0.02) 1px, transparent 1px),
		linear-gradient(90deg, rgba(255, 255, 255, 0.02) 1px, transparent 1px);
	background-size: 50px 50px;
	mask-image: radial-gradient(ellipse at center, black 30%, transparent 70%);
	-webkit-mask-image: radial-gradient(ellipse at center, black 30%, transparent 70%);
}

.welcome-content {
	position: relative;
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 2rem;
	padding: 2rem;
}

.logo-wrapper {
	position: relative;
	width: 8rem;
	height: 8rem;
}

.logo-glow {
	width: 100%;
	height: 100%;
	filter: drop-shadow(0 0 24px rgba(31, 107, 79, 0.35));
}

.welcome-title {
	margin: 0;
	font-size: 2.5rem;
	font-weight: 700;
	letter-spacing: -0.04em;
	line-height: 1;
	color: white;
	animation: title-in 0.8s cubic-bezier(0.32, 0.72, 0, 1) both;
}

@keyframes title-in {
	from {
		opacity: 0;
		transform: translateY(20px);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}

.welcome-subtitle {
	margin: 0;
	font-size: 1.125rem;
	color: rgba(255, 255, 255, 0.6);
	animation: title-in 0.8s cubic-bezier(0.32, 0.72, 0, 1) 0.1s both;
}

.welcome-actions {
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 1.25rem;
	width: 20rem;
	animation: title-in 0.8s cubic-bezier(0.32, 0.72, 0, 1) 0.2s both;
}

.pack-card-wrapper {
	width: 100%;
}

.welcome-cta {
	width: 100%;
	font-weight: 600;
	padding: 1rem 2rem;
	font-size: 1rem;
}

.hint-text {
	display: flex;
	align-items: center;
	gap: 0.25rem;
	font-size: 0.875rem;
	color: rgba(255, 255, 255, 0.4);
}

.shortcut-key {
	display: inline-flex;
	height: 1.5rem;
	min-width: 1.5rem;
	align-items: center;
	justify-content: center;
	border-radius: 0.375rem;
	border: 1px solid rgba(255, 255, 255, 0.1);
	background: rgba(255, 255, 255, 0.05);
	padding: 0 0.5rem;
	font-size: 0.75rem;
	font-weight: 500;
	color: rgba(255, 255, 255, 0.7);
}

.welcome-footer {
	position: relative;
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 1rem;
	padding: 2rem;
	font-size: 0.875rem;
	color: rgba(255, 255, 255, 0.5);
}

.import-btn {
	background: rgba(255, 255, 255, 0.05) !important;
	border: 1px solid rgba(255, 255, 255, 0.1) !important;
	backdrop-filter: blur(10px);
	-webkit-backdrop-filter: blur(10px);
	transition: all 0.2s ease;

	&:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.1) !important;
		border-color: rgba(31, 107, 79, 0.5) !important;
	}
}
</style>
