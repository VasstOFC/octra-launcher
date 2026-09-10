<script setup lang="ts">
import { PlayIcon, SpinnerIcon, StopCircleIcon } from '@lumen/assets'
import {
	Avatar,
	Button,
	defineMessages,
	injectNotificationManager,
	useRelativeTime,
	useVIntl,
} from '@lumen/ui'
import dayjs from 'dayjs'
import { computed, inject, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instance: GameInstance | null
}>()

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ numeric: 'auto', style: 'short' })
const router = useRouter()
const showCreationModal = inject<() => void>('showCreationModal')

const playing = ref(false)
const loading = ref(false)
const currentEvent = ref<'installing' | 'launched' | 'finished' | null>(null)

const messages = defineMessages({
	continue: {
		id: 'app.home.continue.title',
		defaultMessage: 'Continue',
	},
	play: {
		id: 'app.home.continue.play',
		defaultMessage: 'Play',
	},
	stop: {
		id: 'app.home.continue.stop',
		defaultMessage: 'Stop',
	},
	loading: {
		id: 'app.home.continue.loading',
		defaultMessage: 'Starting…',
	},
	played: {
		id: 'app.home.continue.played',
		defaultMessage: 'Last played {relativeTime}',
	},
	neverPlayed: {
		id: 'app.home.continue.never-played',
		defaultMessage: 'Ready to play',
	},
	openInstance: {
		id: 'app.home.continue.open',
		defaultMessage: 'Open {name}',
	},
	empty: {
		id: 'app.home.continue.empty',
		defaultMessage: 'Create an instance to start playing.',
	},
	emptyCta: {
		id: 'app.home.continue.empty-cta',
		defaultMessage: 'Create instance',
	},
})

const iconSrc = computed(() =>
	props.instance ? getInstanceIconUrl(props.instance.icon_path) : undefined,
)
const installing = computed(() => props.instance?.install_stage.includes('installing') ?? false)
const modLoading = computed(
	() =>
		loading.value ||
		currentEvent.value === 'installing' ||
		(currentEvent.value === 'launched' && !playing.value),
)
const statusLine = computed(() => {
	if (!props.instance) return formatMessage(messages.empty)
	if (props.instance.last_played) {
		return formatMessage(messages.played, {
			relativeTime: formatRelativeTime(dayjs(props.instance.last_played).toISOString()),
		})
	}
	return formatMessage(messages.neverPlayed)
})
const metaLine = computed(() => {
	if (!props.instance) return ''
	return `${props.instance.loader} ${props.instance.game_version}`
})

async function checkProcess() {
	if (!props.instance) {
		playing.value = false
		return
	}
	const runningProcesses = (await get_by_instance_id(props.instance.id).catch(handleError)) ?? []
	playing.value = runningProcesses.length > 0
}

async function play() {
	if (!props.instance || props.instance.quarantined) return
	loading.value = true
	await run(props.instance.id)
		.catch((err) => handleSevereError(err, { instanceId: props.instance!.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: props.instance!.loader,
				game_version: props.instance!.game_version,
				source: 'HomeContinue',
			})
		})
	loading.value = false
}

async function stop() {
	if (!props.instance) return
	playing.value = false
	await kill(props.instance.id).catch(handleError)
	trackEvent('InstanceStop', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		source: 'HomeContinue',
	})
}

function openInstance() {
	if (!props.instance) return
	void router.push(`/instance/${encodeURIComponent(props.instance.id)}`)
}

useAppEvent('process', (event) => {
	if (props.instance && event.instance_id === props.instance.id) {
		currentEvent.value = event.event
		playing.value = event.event === 'launched'
	}
})

watch(
	() => props.instance?.id,
	() => {
		currentEvent.value = null
		void checkProcess()
	},
)

onMounted(() => {
	void checkProcess()
})
</script>

<template>
	<section class="continue-band" :aria-label="formatMessage(messages.continue)">
		<div v-if="instance" class="continue-band__row">
			<button
				type="button"
				class="continue-band__identity"
				:aria-label="formatMessage(messages.openInstance, { name: instance.name })"
				@click="openInstance"
			>
				<Avatar
					class="!rounded-lg shrink-0"
					size="64px"
					:src="iconSrc"
					:tint-by="instance.id"
					alt=""
					no-shadow
					pad-transparent-corners
				/>
				<div class="min-w-0 flex flex-col gap-1 text-left">
					<p
						class="continue-band__kicker m-0 truncate text-xs font-semibold uppercase tracking-wide"
					>
						{{ formatMessage(messages.continue) }}
					</p>
					<h2 class="m-0 truncate text-2xl font-semibold leading-7 text-contrast">
						{{ instance.name }}
					</h2>
					<p class="m-0 truncate text-sm capitalize leading-5 text-primary">
						{{ metaLine }}
						<span class="text-secondary"> · {{ statusLine }}</span>
					</p>
				</div>
			</button>
			<div class="continue-band__actions">
				<Button
					v-if="playing"
					type="colored"
					color="red"
					size="lg"
					class="!shadow-none band-btn--stop"
					@click="stop"
				>
					<StopCircleIcon />
					{{ formatMessage(messages.stop) }}
				</Button>
				<Button
					v-else-if="modLoading || installing"
					type="colored"
					color="brand"
					size="lg"
					class="!shadow-none"
					disabled
				>
					<SpinnerIcon class="animate-spin" />
					{{ formatMessage(messages.loading) }}
				</Button>
				<Button
					v-else-if="!instance.quarantined"
					type="colored"
					color="brand"
					size="lg"
					class="!shadow-none band-btn--play"
					@click="play"
					@mouseenter="checkProcess"
				>
					<PlayIcon class="translate-x-px" />
					{{ formatMessage(messages.play) }}
				</Button>
			</div>
		</div>
		<p v-else class="m-0 flex flex-wrap items-center gap-3 text-sm text-secondary">
			<span>{{ formatMessage(messages.empty) }}</span>
			<Button
				type="colored"
				color="brand"
				size="sm"
				class="!shadow-none"
				@click="showCreationModal?.()"
			>
				{{ formatMessage(messages.emptyCta) }}
			</Button>
		</p>
	</section>
</template>

<style scoped lang="scss">
.continue-band {
	position: relative;
	border: 1px solid rgba(0, 212, 255, 0.12);
	border-radius: var(--radius-lg);
	margin: 0;
	padding: 2rem;
	background:
		linear-gradient(
			120deg,
			rgba(0, 212, 255, 0.08) 0%,
			transparent 52%
		),
		rgba(15, 15, 25, 0.85);
	backdrop-filter: blur(20px);
	-webkit-backdrop-filter: blur(20px);
	box-shadow:
		0 0 0 1px rgba(0, 212, 255, 0.06),
		inset 0 2px 0 0 rgba(0, 212, 255, 0.2),
		0 20px 40px -15px rgba(0, 0, 0, 0.4);
	overflow: hidden;
	transition: all 0.3s ease;

	&::before {
		content: '';
		position: absolute;
		inset: 0;
		background: radial-gradient(
			ellipse at 20% 50%,
			rgba(0, 212, 255, 0.06) 0%,
			transparent 60%
		);
		pointer-events: none;
	}

	&:hover {
		border-color: rgba(0, 212, 255, 0.18);
		box-shadow:
			0 0 0 1px rgba(0, 212, 255, 0.1),
			inset 0 2px 0 0 rgba(0, 212, 255, 0.3),
			0 25px 50px -15px rgba(0, 0, 0, 0.5);
	}
}

.continue-band__kicker {
	color: #00d4ff;
	text-shadow: 0 0 10px rgba(0, 212, 255, 0.4);
	letter-spacing: 0.08em;
}

.continue-band__row {
	align-items: center;
	display: flex;
	flex-wrap: wrap;
	gap: 1.25rem;
	justify-content: space-between;
	min-width: 0;
}

.continue-band__identity {
	align-items: center;
	background: transparent;
	border: 0;
	cursor: pointer;
	display: flex;
	gap: 1rem;
	min-width: 0;
	padding: 0;
	text-align: left;
	transition: transform 0.2s ease;

	&:hover {
		transform: translateX(4px);
	}

	&:focus-visible {
		outline: 2px solid #00d4ff;
		outline-offset: 2px;
		border-radius: var(--radius-sm);
	}
}

.continue-band__actions {
	display: flex;
	flex-shrink: 0;
	gap: 0.5rem;
}

.continue-band__actions :deep(.btn) {
	min-width: 7.5rem;
}

.band-btn--play {
	background: linear-gradient(135deg, #00d4ff 0%, #7c3aed 100%) !important;
	border: none !important;
	box-shadow: 0 0 0 rgba(0, 212, 255, 0) !important;
	transition: all 0.2s ease;

	&:hover:not(:disabled) {
		transform: translateY(-2px) !important;
		box-shadow: 0 0 25px rgba(0, 212, 255, 0.4), 0 0 50px rgba(0, 212, 255, 0.15) !important;
		animation: play-glow 1.5s ease-in-out infinite;
	}
}

@keyframes play-glow {
	0%, 100% {
		box-shadow: 0 0 20px rgba(0, 212, 255, 0.4), 0 0 40px rgba(0, 212, 255, 0.1);
	}
	50% {
		box-shadow: 0 0 30px rgba(0, 212, 255, 0.6), 0 0 60px rgba(0, 212, 255, 0.2);
	}
}

.band-btn--stop {
	transition: all 0.2s ease;

	&:hover:not(:disabled) {
		transform: translateY(-2px) !important;
		box-shadow: 0 0 20px rgba(239, 68, 68, 0.4) !important;
	}
}
</style>
