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
				'Czat Octra: DM-y, grupy, reakcje, przypięte wiadomości i zaproszenia do grup',
				'Lista znajomych z obecnością, „Graj z” i udostępnianiem IP serwera',
				'Wspólne serwery w panelu znajomych oraz podgląd paczek .mrpack przed instalacją',
				'Odznaka nieprzeczytanych wiadomości i powiadomienia o nowych wiadomościach',
			]
		: [
				'Octra chat: DMs, groups, reactions, pinned messages, and group invites',
				'Friends list with presence, Play with, and share-your-server-IP',
				'Shared servers in the friends sidebar plus .mrpack preview before install',
				'Unread chat badge and toasts when new messages arrive',
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
		defaultMessage: 'Social sprint highlights in this build:',
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
