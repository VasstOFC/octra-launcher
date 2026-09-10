<script setup lang="ts">
import { Button, defineMessages, NewModal, useVIntl } from '@lumen/ui'
import { ref } from 'vue'

const LAST_SEEN_KEY = 'Lumen-last-seen-version'

const props = defineProps<{
	version: string
}>()

const emit = defineEmits<{
	dismissed: []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)

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
		id: 'Lumen.whats-new.title',
		defaultMessage: "What's new in Lumen {version}",
	},
	dismiss: {
		id: 'Lumen.whats-new.dismiss',
		defaultMessage: 'Got it',
	},
	fallback: {
		id: 'Lumen.whats-new.fallback',
		defaultMessage: 'See the full changelog on GitHub for details.',
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
		<p class="m-0 text-sm text-primary">
			{{ formatMessage(messages.fallback) }}
		</p>
		<template #actions>
			<div class="flex justify-end">
				<Button type="colored" color="brand" @click="dismiss">
					{{ formatMessage(messages.dismiss) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
