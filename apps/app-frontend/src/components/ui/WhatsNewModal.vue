<script setup lang="ts">
import { Button, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

const LAST_SEEN_KEY = 'octra-last-seen-version'

const props = defineProps<{
	version: string
}>()

const emit = defineEmits<{
	dismissed: []
}>()

const { formatMessage, locale } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)

const isPolish = computed(() =>
	String(locale.value || '')
		.toLowerCase()
		.startsWith('pl'),
)

	const featureBullets = computed(() =>
	isPolish.value
		? [
				'Play Dock: szeroki dock biblioteki i nawigacja Start / Serwery / Szafka / Odkrywaj',
				'Odkrywaj: cichsze karty, szuflada filtrów, skróty sortowania i siatka modpacków',
				'Serwery: wspólne vs lokalne, udostępnianie IP, usuwanie dla wszystkich i live sync',
				'Czat w panelu znajomych: grupy, reakcje, screenshoty i znaczniki czasu',
				'Domyślny akcent lawendowy oraz gęstsze, grafitowe ustawienia',
			]
		: [
				'Play Dock: wide library dock and Start / Servers / Locker / Discover navigation',
				'Discover: quieter cards, filter drawer, sort shortcuts, and modpack grid',
				'Servers: shared vs local lists, share IP, delete-for-everyone, and live sync',
				'Chat in the friends panel: groups, reactions, screenshots, and timestamps',
				'Default lavender accent plus denser graphite settings',
			],
)

function shouldShow(version: string): boolean {
	if (!version) return false
	try {
		return localStorage.getItem(LAST_SEEN_KEY) !== version
	} catch {
		return false
	}
}

function markSeen() {
	try {
		localStorage.setItem(LAST_SEEN_KEY, props.version)
	} catch {
		// ignore storage failures
	}
}

function show() {
	if (!shouldShow(props.version)) return
	modal.value?.show()
}

function dismiss() {
	markSeen()
	modal.value?.hide()
	emit('dismissed')
}

defineExpose({ show, shouldShow })

const messages = defineMessages({
	title: {
		id: 'octra.whats-new.title',
		defaultMessage: "What's new in Octra {version}",
	},
	intro: {
		id: 'octra.whats-new.intro',
		defaultMessage: 'Highlights since your last update:',
	},
	dismiss: {
		id: 'octra.whats-new.dismiss',
		defaultMessage: 'Got it',
	},
})
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title, { version })"
		max-width="440px"
		:on-hide="markSeen"
	>
		<p class="m-0 mb-3 text-sm text-secondary">
			{{ formatMessage(messages.intro) }}
		</p>
		<ul class="m-0 flex list-disc flex-col gap-2 pl-5 text-sm text-primary">
			<li v-for="(bullet, index) in featureBullets" :key="index">
				{{ bullet }}
			</li>
		</ul>
		<template #actions>
			<div class="flex justify-end">
				<Button type="colored" color="brand" @click="dismiss">
					{{ formatMessage(messages.dismiss) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
