<script setup lang="ts">
import { MessageIcon } from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, ref, useTemplateRef } from 'vue'

import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { handleSevereError } from '@/composables/use-error.js'
import type { InstanceScreenshot } from '@/helpers/instance'
import {
	octraAccountSession,
	octraChatOpenDm,
	octraChatPost,
	octraChatUploadImage,
	octraCommunity,
} from '@/helpers/octra-account.js'

type CommunityMember = {
	id: number
	minecraft_nick: string
	profile_uuid: string
	avatar_url: string
}

const emit = defineEmits<{
	shared: [userId: number]
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const openOctraChatDm = inject<(userId: number) => void | Promise<void>>('openOctraChatDm', () => {})

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const screenshot = ref<InstanceScreenshot | null>(null)
const members = ref<CommunityMember[]>([])
const loading = ref(false)
const sendingId = ref<number | null>(null)
const signedIn = ref(false)

const { avatarFor } = useOctraCommunityAvatars(members)

const sortedMembers = computed(() =>
	members.value
		.slice()
		.sort((a, b) =>
			a.minecraft_nick.localeCompare(b.minecraft_nick, undefined, { sensitivity: 'base' }),
		),
)

const messages = defineMessages({
	title: {
		id: 'app.screenshots.share.title',
		defaultMessage: 'Share with a friend',
	},
	hint: {
		id: 'app.screenshots.share.hint',
		defaultMessage: 'Pick someone to send this screenshot in Octra chat.',
	},
	signIn: {
		id: 'app.screenshots.share.sign-in',
		defaultMessage: 'Sign in to Octra to share screenshots with friends.',
	},
	empty: {
		id: 'app.screenshots.share.empty',
		defaultMessage: 'No friends found yet.',
	},
	chatBody: {
		id: 'app.screenshots.share.chat-body',
		defaultMessage: 'Shared a screenshot: {name} ({instance})',
	},
	success: {
		id: 'app.screenshots.share.success',
		defaultMessage: 'Screenshot shared with {nick}',
	},
})

async function show(target: InstanceScreenshot) {
	screenshot.value = target
	sendingId.value = null
	loading.value = true
	modal.value?.show()
	try {
		const session = await octraAccountSession()
		signedIn.value = !!session
		if (!session) {
			members.value = []
			return
		}
		const snap = await octraCommunity()
		members.value = (snap?.members ?? []).map((member) => ({
			id: member.id,
			minecraft_nick: member.minecraft_nick,
			profile_uuid: member.profile_uuid,
			avatar_url: member.avatar_url || '',
		}))
	} catch (error) {
		signedIn.value = false
		members.value = []
		handleSevereError(error, handleError)
	} finally {
		loading.value = false
	}
}

async function shareWith(member: CommunityMember) {
	if (!screenshot.value || sendingId.value != null) return
	sendingId.value = member.id
	try {
		const channel = await octraChatOpenDm(member.id)
		const uploaded = await octraChatUploadImage(screenshot.value.path)
		const body = formatMessage(messages.chatBody, {
			name: screenshot.value.file_name,
			instance: screenshot.value.instance_name || '—',
		})
		await octraChatPost(channel.id, body, uploaded.path)
		await openOctraChatDm(member.id)
		addNotification({
			type: 'success',
			title: formatMessage(messages.success, { nick: member.minecraft_nick }),
		})
		emit('shared', member.id)
		modal.value?.hide()
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		sendingId.value = null
	}
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" class="w-[min(24rem,92vw)]">
		<p class="m-0 mb-3 text-sm text-secondary">
			{{ formatMessage(signedIn ? messages.hint : messages.signIn) }}
		</p>
		<div v-if="loading" class="flex flex-col gap-2">
			<div v-for="n in 4" :key="n" class="flex animate-pulse items-center gap-2">
				<div class="size-8 rounded-full bg-button-bg" />
				<div class="h-3 w-1/2 rounded-full bg-button-bg" />
			</div>
		</div>
		<p v-else-if="!signedIn" class="m-0 text-sm text-secondary">
			{{ formatMessage(messages.signIn) }}
		</p>
		<p v-else-if="sortedMembers.length === 0" class="m-0 text-sm text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div v-else class="flex max-h-72 flex-col overflow-y-auto">
			<button
				v-for="member in sortedMembers"
				:key="member.id"
				type="button"
				class="flex items-center gap-2 border-0 bg-transparent px-1 py-2 text-left text-sm text-primary cursor-pointer hover:bg-button-bg disabled:opacity-50"
				:disabled="sendingId != null"
				@click="shareWith(member)"
			>
				<Avatar :src="avatarFor(member)" :alt="member.minecraft_nick" size="32px" circle />
				<span class="min-w-0 flex-1 truncate font-medium text-contrast">{{
					member.minecraft_nick
				}}</span>
				<MessageIcon class="size-4 shrink-0 text-secondary" />
			</button>
		</div>
	</NewModal>
</template>
