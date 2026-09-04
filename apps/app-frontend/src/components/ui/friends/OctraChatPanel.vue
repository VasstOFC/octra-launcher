<script setup lang="ts">
import {
	ChevronLeftIcon,
	DownloadIcon,
	MessageIcon,
	PinIcon,
	PlusIcon,
	SendIcon,
	TrashIcon,
	UsersIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	type ButtonMenuOption,
	commonMessages,
	ContextMenu,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, nextTick, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import {
	OCTRA_COMMUNITY_STEVE_HEAD,
	useOctraCommunityAvatars,
} from '@/composables/use-octra-community-avatars'
import {
	install_create_modpack_instance,
	install_get_modpack_preview,
	type InstallModpackPreview,
} from '@/helpers/install'
import {
	channelTitle,
	extractMrpackUrls,
	octraCacheMrpackUrl,
	octraChatAddMembers,
	octraChatCastDeleteVote,
	octraChatChannels,
	octraChatCreateGroup,
	octraChatDeleteMessage,
	octraChatGetDeleteVote,
	octraChatList,
	octraChatMarkRead,
	octraChatOpenDm,
	octraChatPinMessage,
	octraChatPost,
	octraChatReactMessage,
	octraCommunity,
} from '@/helpers/octra-account.js'

type OctraAccountSession = {
	token: string
	username: string
	minecraft_nick: string
	profile_uuid: string
}

type ChatMember = {
	id: number
	minecraft_nick: string
	profile_uuid: string
}

type ChatReaction = {
	emoji: string
	count: number
	user_ids?: number[]
}

type ChatDeleteVote = {
	active: boolean
	channel_id: number
	member_count: number
	yes_count: number
	no_count: number
	needed: number
	my_vote?: boolean | null
	deleted?: boolean
}

type ChatChannel = {
	id: number
	kind: string
	name?: string | null
	created_at: string
	last_body?: string | null
	last_at?: string | null
	last_id?: number | null
	last_read_id?: number
	unread_count?: number
	members?: ChatMember[]
}

type ChatMessage = {
	id: number
	channel_id?: number
	user_id: number
	minecraft_nick: string
	body: string
	created_at: string
	pinned?: boolean
	deleted?: boolean
	attachment_url?: string | null
	reactions?: ChatReaction[]
}

type CommunityMember = {
	id: number
	minecraft_nick: string
	profile_uuid: string
	avatar_url?: string
}

type AvatarPerson = {
	profile_uuid: string
	minecraft_nick: string
	avatar_url: string
}

type BubbleRow = {
	message: ChatMessage
	mine: boolean
	showName: boolean
	showAvatar: boolean
	clustered: boolean
}

type MrpackPreviewState = {
	url: string
	path: string
	preview: InstallModpackPreview | null
	loading: boolean
}

const REACTION_EMOJIS = ['👍', '❤️', '😂', '🔥', '🎉', '👀'] as const

const props = defineProps<{
	session: OctraAccountSession | null
	open: boolean
	/** Render inside Friends sidebar (single-column, no floating rail chrome). */
	embedded?: boolean
}>()

const emit = defineEmits<{
	close: []
	signIn: []
	unreadChanged: [total: number]
}>()

const embedded = computed(() => !!props.embedded)

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const channels = ref<ChatChannel[]>([])
const community = ref<CommunityMember[]>([])
const selectedId = ref<number | null>(null)
const messagesList = ref<ChatMessage[]>([])
const draft = ref('')
const sending = ref(false)
const installingUrl = ref<string | null>(null)
const mrpackPreview = ref<MrpackPreviewState | null>(null)
const scrollEl = ref<HTMLElement | null>(null)
const showNewDm = ref(false)
const showNewGroup = ref(false)
const showInvite = ref(false)
const showDeleteVote = ref(false)
const groupName = ref('')
const groupPick = ref<number[]>([])
const invitePick = ref<number[]>([])
const deleteVote = ref<ChatDeleteVote | null>(null)
const deleteVoteBusy = ref(false)
const hoveredMessageId = ref<number | null>(null)
const messageOptions = useTemplateRef<InstanceType<typeof ContextMenu>>('messageOptions')
let pollTimer: ReturnType<typeof setInterval> | null = null

const SKINS_FALLBACK_ORIGIN = 'http://92.5.186.6'
const MIN_GROUP_MEMBERS = 3

const selected = computed(() => channels.value.find((c) => c.id === selectedId.value) ?? null)

const showChannelList = computed(() => !embedded.value || selectedId.value == null)
const showThreadPane = computed(() => !embedded.value || selectedId.value != null)

const unreadTotal = computed(() =>
	channels.value.reduce((sum, channel) => sum + (channel.unread_count ?? 0), 0),
)

const selfUserId = computed(() => {
	const uuid = props.session?.profile_uuid?.toLowerCase()
	const nick = props.session?.minecraft_nick?.toLowerCase()
	const pools = [
		...(selected.value?.members || []),
		...channels.value.flatMap((c) => c.members || []),
	]
	for (const member of pools) {
		if (uuid && member.profile_uuid?.toLowerCase() === uuid) return member.id
		if (nick && member.minecraft_nick?.toLowerCase() === nick) return member.id
	}
	return null
})

const canInvite = computed(() => !!selected.value && selected.value.kind === 'group')

const canCreateGroup = computed(
	() =>
		groupName.value.trim().length > 0 &&
		groupPick.value.length + 1 >= MIN_GROUP_MEMBERS,
)

const inviteCandidates = computed(() => {
	const memberIds = new Set((selected.value?.members || []).map((m) => m.id))
	return community.value.filter((m) => !memberIds.has(m.id))
})

const lastId = computed(() =>
	messagesList.value.length > 0 ? messagesList.value[messagesList.value.length - 1]!.id : 0,
)

const skinsOrigin = computed(() => {
	for (const member of community.value) {
		if (!member.avatar_url) continue
		try {
			return new URL(member.avatar_url).origin
		} catch {
			// ignore
		}
	}
	return SKINS_FALLBACK_ORIGIN
})

const avatarPeople = computed<AvatarPerson[]>(() => {
	const byNick = new Map<string, AvatarPerson>()
	const origin = skinsOrigin.value
	const push = (nick: string, uuid = '', avatarUrl = '') => {
		const key = nick.trim().toLowerCase()
		if (!key || byNick.has(key)) return
		byNick.set(key, {
			minecraft_nick: nick,
			profile_uuid: uuid,
			avatar_url: avatarUrl || `${origin}/skins/MinecraftSkins/${encodeURIComponent(nick)}.png`,
		})
	}

	if (props.session) {
		push(props.session.minecraft_nick, props.session.profile_uuid)
	}
	for (const member of community.value) {
		push(member.minecraft_nick, member.profile_uuid, member.avatar_url || '')
	}
	for (const channel of channels.value) {
		for (const member of channel.members || []) {
			push(member.minecraft_nick, member.profile_uuid)
		}
	}
	for (const message of messagesList.value) {
		push(message.minecraft_nick)
	}
	return [...byNick.values()]
})

const { avatarFor } = useOctraCommunityAvatars(avatarPeople)

function headForNick(nick: string): string {
	const person = avatarPeople.value.find(
		(p) => p.minecraft_nick.toLowerCase() === nick.trim().toLowerCase(),
	)
	if (!person) return OCTRA_COMMUNITY_STEVE_HEAD
	return avatarFor(person)
}

function isMine(message: ChatMessage) {
	const self = props.session?.minecraft_nick?.toLowerCase()
	return !!self && message.minecraft_nick.toLowerCase() === self
}

const bubbles = computed<BubbleRow[]>(() => {
	const list = messagesList.value
	return list.map((message, index) => {
		const prev = list[index - 1]
		const next = list[index + 1]
		const mine = isMine(message)
		const sameAsPrev = !!prev && prev.user_id === message.user_id
		const sameAsNext = !!next && next.user_id === message.user_id
		return {
			message,
			mine,
			showName: !mine && !sameAsPrev,
			// Discord/Messenger: head only on the last bubble of a same-sender streak (both sides).
			showAvatar: !sameAsNext,
			clustered: sameAsPrev,
		}
	})
})

function titleFor(channel: ChatChannel) {
	return channelTitle(channel, props.session?.minecraft_nick)
}

function formatTime(value: string) {
	return new Date(value).toLocaleString([], {
		dateStyle: 'medium',
		timeStyle: 'short',
	})
}

function openMessageContextMenu(row: BubbleRow, event: MouseEvent) {
	if (row.message.deleted || !row.mine) return
	const options: ButtonMenuOption[] = [
		{
			id: 'delete',
			label: formatMessage(messages.delete),
			icon: TrashIcon,
			tone: 'red',
			action: () => void deleteMessage(row.message),
		},
	]
	messageOptions.value?.open(event, options)
}

async function scrollToBottom() {
	await nextTick()
	if (scrollEl.value) {
		scrollEl.value.scrollTop = scrollEl.value.scrollHeight
	}
}

function emitUnread() {
	emit('unreadChanged', unreadTotal.value)
}

async function markChannelRead(channelId: number, maxId: number) {
	if (maxId <= 0) return
	try {
		await octraChatMarkRead(channelId, maxId)
		const channel = channels.value.find((c) => c.id === channelId)
		if (channel) {
			channel.unread_count = 0
			channel.last_read_id = maxId
		}
		emitUnread()
	} catch {
		// ignore mark-read failures
	}
}

async function refreshChannels() {
	if (!props.session) {
		channels.value = []
		emitUnread()
		return
	}
	const rows = await octraChatChannels()
	channels.value = rows.filter(
		(c) => !(c.kind === 'group' && (c.name || '').toLowerCase() === 'everyone'),
	)
	if (selectedId.value != null && !channels.value.some((c) => c.id === selectedId.value)) {
		selectedId.value = null
		messagesList.value = []
		deleteVote.value = null
		showDeleteVote.value = false
	}
	if (!embedded.value && selectedId.value == null && channels.value.length > 0) {
		selectedId.value = channels.value[0]!.id
	}
	emitUnread()
}

async function refreshCommunity() {
	if (!props.session) {
		community.value = []
		return
	}
	const snap = await octraCommunity()
	community.value = snap?.members ?? []
}

async function loadMessages(reset = false) {
	if (!props.session || selectedId.value == null) {
		messagesList.value = []
		return
	}
	const channelId = selectedId.value
	const rows = await octraChatList(channelId, reset ? 0 : lastId.value)
	if (reset) {
		messagesList.value = rows
		await scrollToBottom()
	} else if (rows.length > 0) {
		const known = new Set(messagesList.value.map((m) => m.id))
		const appended = rows.filter((m) => !known.has(m.id))
		if (appended.length > 0) {
			messagesList.value = [...messagesList.value, ...appended]
			await scrollToBottom()
		}
	}
	const maxId = messagesList.value.reduce((max, m) => Math.max(max, m.id), 0)
	if (maxId > 0) {
		await markChannelRead(channelId, maxId)
	}
}

function stopPolling() {
	if (pollTimer) {
		clearInterval(pollTimer)
		pollTimer = null
	}
}

function startPolling() {
	stopPolling()
	pollTimer = setInterval(() => {
		void loadMessages(false)
		void refreshChannels().catch(() => undefined)
		if (selected.value?.kind === 'group') {
			void refreshDeleteVote()
		}
	}, 8_000)
}

watch(
	() => [props.open, props.session?.username ?? null] as const,
	async ([isOpen]) => {
		stopPolling()
		if (!isOpen || !props.session) {
			messagesList.value = []
			if (embedded.value) {
				selectedId.value = null
			}
			return
		}
		try {
			await Promise.all([refreshChannels(), refreshCommunity()])
			if (selectedId.value != null) {
				await loadMessages(true)
			}
			startPolling()
		} catch (error) {
			handleSevereError(error, handleError)
		}
	},
	{ immediate: true },
)

watch(selectedId, async () => {
	if (!props.open || !props.session || selectedId.value == null) {
		deleteVote.value = null
		showDeleteVote.value = false
		return
	}
	showInvite.value = false
	invitePick.value = []
	showDeleteVote.value = false
	mrpackPreview.value = null
	try {
		await loadMessages(true)
		await refreshDeleteVote()
	} catch (error) {
		handleSevereError(error, handleError)
	}
})

function backToList() {
	selectedId.value = null
	messagesList.value = []
	showInvite.value = false
	invitePick.value = []
	mrpackPreview.value = null
}

onUnmounted(() => stopPolling())

async function send() {
	const text = draft.value.trim()
	if (!text || selectedId.value == null || sending.value) return
	sending.value = true
	try {
		const posted = await octraChatPost(selectedId.value, text)
		if (!messagesList.value.some((m) => m.id === posted.id)) {
			messagesList.value = [...messagesList.value, posted]
		}
		draft.value = ''
		await refreshChannels()
		await scrollToBottom()
		await markChannelRead(selectedId.value, posted.id)
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		sending.value = false
	}
}

async function openDm(userId: number) {
	try {
		const channel = await octraChatOpenDm(userId)
		await refreshChannels()
		selectedId.value = channel.id
		showNewDm.value = false
		showNewGroup.value = false
		await loadMessages(true)
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

defineExpose({ openDm, unreadTotal })

async function createGroup() {
	const name = groupName.value.trim()
	if (!name || !canCreateGroup.value) return
	try {
		const channel = await octraChatCreateGroup(name, groupPick.value)
		await refreshChannels()
		selectedId.value = channel.id
		groupName.value = ''
		groupPick.value = []
		showNewGroup.value = false
		await loadMessages(true)
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

function reactionIsMine(reaction: ChatReaction) {
	const uid = selfUserId.value
	if (uid == null) return false
	return (reaction.user_ids || []).includes(uid)
}

function applyMessageUpdate(updated: ChatMessage) {
	const index = messagesList.value.findIndex((m) => m.id === updated.id)
	if (index >= 0) {
		messagesList.value[index] = { ...messagesList.value[index], ...updated }
	}
}

function openAttachment(url: string) {
	void openUrl(url)
}

function channelPreview(channel: ChatChannel) {
	const body = channel.last_body?.trim()
	if (!body) return '—'
	if (body === '[image]') return formatMessage(messages.imageAttachment)
	return body
}

async function refreshDeleteVote() {
	if (!selected.value || selected.value.kind !== 'group') {
		deleteVote.value = null
		return
	}
	try {
		deleteVote.value = await octraChatGetDeleteVote(selected.value.id)
	} catch {
		deleteVote.value = null
	}
}

async function castDeleteVote(yes: boolean) {
	if (!selected.value || selected.value.kind !== 'group' || deleteVoteBusy.value) return
	deleteVoteBusy.value = true
	try {
		const result = await octraChatCastDeleteVote(selected.value.id, yes)
		deleteVote.value = result
		if (result.deleted) {
			showDeleteVote.value = false
			selectedId.value = null
			messagesList.value = []
			await refreshChannels()
		}
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		deleteVoteBusy.value = false
	}
}

function toggleNewDmPanel() {
	showNewDm.value = !showNewDm.value
	showNewGroup.value = false
}

function toggleNewGroupPanel() {
	showNewGroup.value = !showNewGroup.value
	showNewDm.value = false
}

function toggleInvitePanel() {
	showInvite.value = !showInvite.value
	showDeleteVote.value = false
}

function toggleDeleteVotePanel() {
	showDeleteVote.value = !showDeleteVote.value
	showInvite.value = false
	if (showDeleteVote.value) {
		void refreshDeleteVote()
	}
}

function toggleGroupMember(id: number) {
	if (groupPick.value.includes(id)) {
		groupPick.value = groupPick.value.filter((x) => x !== id)
	} else {
		groupPick.value = [...groupPick.value, id]
	}
}

function toggleInviteMember(id: number) {
	if (invitePick.value.includes(id)) {
		invitePick.value = invitePick.value.filter((x) => x !== id)
	} else {
		invitePick.value = [...invitePick.value, id]
	}
}

async function inviteMembers() {
	if (!selectedId.value || invitePick.value.length === 0) return
	try {
		await octraChatAddMembers(selectedId.value, invitePick.value)
		invitePick.value = []
		showInvite.value = false
		await refreshChannels()
		await refreshDeleteVote()
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function reactToMessage(message: ChatMessage, emoji: string) {
	try {
		const updated = await octraChatReactMessage(message.id, emoji)
		applyMessageUpdate(updated)
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function togglePin(message: ChatMessage) {
	try {
		await octraChatPinMessage(message.id, !message.pinned)
		message.pinned = !message.pinned
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function deleteMessage(message: ChatMessage) {
	if (!isMine(message)) return
	try {
		await octraChatDeleteMessage(message.id)
		message.deleted = true
		message.body = ''
		message.attachment_url = null
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function beginMrpackInstall(url: string) {
	if (installingUrl.value || mrpackPreview.value?.loading) return
	mrpackPreview.value = { url, path: '', preview: null, loading: true }
	try {
		const path = await octraCacheMrpackUrl(url)
		const preview = await install_get_modpack_preview({ type: 'fromFile', path })
		mrpackPreview.value = { url, path, preview, loading: false }
	} catch (error) {
		mrpackPreview.value = null
		handleSevereError(error, handleError)
	}
}

async function confirmMrpackInstall() {
	const state = mrpackPreview.value
	if (!state?.path || state.loading) return
	installingUrl.value = state.url
	try {
		await install_create_modpack_instance({ type: 'fromFile', path: state.path })
		mrpackPreview.value = null
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		installingUrl.value = null
	}
}

function cancelMrpackInstall() {
	mrpackPreview.value = null
}

function onKeydown(event: KeyboardEvent) {
	if (event.key === 'Enter' && !event.shiftKey) {
		event.preventDefault()
		void send()
	}
}

const messages = defineMessages({
	heading: { id: 'octra.chat.heading', defaultMessage: 'Chat' },
	close: { id: 'octra.chat.close', defaultMessage: 'Close chat' },
	placeholder: {
		id: 'octra.chat.placeholder',
		defaultMessage: 'Write a message…',
	},
	send: { id: 'octra.chat.send', defaultMessage: 'Send' },
	installPack: { id: 'octra.chat.install-pack', defaultMessage: 'Install pack' },
	installConfirm: {
		id: 'octra.chat.install-confirm',
		defaultMessage: 'Install',
	},
	installCancel: {
		id: 'octra.chat.install-cancel',
		defaultMessage: 'Cancel',
	},
	installPreviewLoading: {
		id: 'octra.chat.install-preview-loading',
		defaultMessage: 'Loading pack preview…',
	},
	installPreviewTitle: {
		id: 'octra.chat.install-preview-title',
		defaultMessage: 'Install {name}?',
	},
	installPreviewMeta: {
		id: 'octra.chat.install-preview-meta',
		defaultMessage: '{gameVersion} · {modloader}',
	},
	signInHint: {
		id: 'octra.chat.sign-in-hint',
		defaultMessage: 'Sign in to Octra to use chat.',
	},
	signInAction: { id: 'octra.chat.sign-in-action', defaultMessage: 'Log in' },
	emptyChannels: {
		id: 'octra.chat.empty-channels',
		defaultMessage: 'No conversations yet.',
	},
	emptyMessages: {
		id: 'octra.chat.empty',
		defaultMessage: 'No messages yet. Say hi or share a pack link.',
	},
	newDm: { id: 'octra.chat.new-dm', defaultMessage: 'Message someone' },
	newGroup: { id: 'octra.chat.new-group', defaultMessage: 'New group' },
	groupName: { id: 'octra.chat.group-name', defaultMessage: 'Group name' },
	createGroup: { id: 'octra.chat.create-group', defaultMessage: 'Create group' },
	pickMembers: {
		id: 'octra.chat.pick-members',
		defaultMessage: 'Add members',
	},
	conversations: {
		id: 'octra.chat.conversations',
		defaultMessage: 'Conversations',
	},
	selectConversation: {
		id: 'octra.chat.select-conversation',
		defaultMessage: 'Select a conversation',
	},
	messageDeleted: {
		id: 'octra.chat.message-deleted',
		defaultMessage: 'Message deleted',
	},
	imageAttachment: {
		id: 'octra.chat.image-attachment',
		defaultMessage: 'Image',
	},
	pin: { id: 'octra.chat.pin', defaultMessage: 'Pin' },
	unpin: { id: 'octra.chat.unpin', defaultMessage: 'Unpin' },
	delete: { id: 'octra.chat.delete', defaultMessage: 'Delete' },
	pinnedBadge: { id: 'octra.chat.pinned', defaultMessage: 'Pinned' },
	inviteMembers: {
		id: 'octra.chat.invite-members',
		defaultMessage: 'Invite members',
	},
	inviteConfirm: {
		id: 'octra.chat.invite-confirm',
		defaultMessage: 'Add to group',
	},
	groupMinMembers: {
		id: 'octra.chat.group-min-members',
		defaultMessage: 'Pick at least 2 other people (3 total including you).',
	},
	deleteGroup: {
		id: 'octra.chat.delete-group',
		defaultMessage: 'Delete group',
	},
	deleteVoteTitle: {
		id: 'octra.chat.delete-vote-title',
		defaultMessage: 'Vote to delete this group',
	},
	deleteVoteHint: {
		id: 'octra.chat.delete-vote-hint',
		defaultMessage: '{yes} of {needed} yes votes needed ({members} members).',
	},
	deleteVoteYes: {
		id: 'octra.chat.delete-vote-yes',
		defaultMessage: 'Vote yes',
	},
	deleteVoteNo: {
		id: 'octra.chat.delete-vote-no',
		defaultMessage: 'Vote no',
	},
	deleteVoteStart: {
		id: 'octra.chat.delete-vote-start',
		defaultMessage: 'Start delete vote',
	},
	deleteVoteConfirm: {
		id: 'octra.chat.delete-vote-confirm',
		defaultMessage: 'A two-thirds majority must vote yes to delete this group.',
	},
	unreadBadge: {
		id: 'octra.chat.unread',
		defaultMessage: '{count} unread',
	},
	backToList: {
		id: 'octra.chat.back-to-list',
		defaultMessage: 'Back to conversations',
	},
	messageActions: {
		id: 'octra.chat.message-actions',
		defaultMessage: 'Message actions',
	},
})
</script>

<template>
	<component
		:is="embedded ? 'div' : 'aside'"
		v-if="open"
		class="relative flex h-full min-h-0 flex-col"
		:class="
			embedded
				? 'octra-chat-embedded w-full min-h-0 flex-1 border-0'
				: 'octra-chat-rail w-[min(560px,52vw)] shrink-0 border-0 border-r border-solid border-surface-5 bg-bg-raised'
		"
	>
		<ContextMenu ref="messageOptions" :label="formatMessage(messages.messageActions)" />
		<div v-if="!embedded" class="octra-chat-rail__accent" aria-hidden="true" />
		<div
			v-if="!embedded"
			class="octra-chat-rail__header flex items-center justify-between gap-2 border-0 border-b border-solid border-surface-5 px-3 py-2"
		>
			<div class="flex items-center gap-2 text-contrast">
				<MessageIcon class="size-4 shrink-0 text-brand" />
				<span class="text-sm font-semibold">{{ formatMessage(messages.heading) }}</span>
			</div>
			<IconButton type="quiet" :label="formatMessage(messages.close)" @click="emit('close')">
				<XIcon />
			</IconButton>
		</div>

		<template v-if="!session">
			<div class="flex flex-col gap-2 p-1">
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.signInHint) }}</p>
				<Button type="colored" color="brand" @click="emit('signIn')">
					{{ formatMessage(messages.signInAction) }}
				</Button>
			</div>
		</template>

		<template v-else>
			<div
				class="flex min-h-0 flex-1"
				:class="embedded ? 'flex-col' : 'flex-row'"
			>
				<div
					v-show="showChannelList"
					class="flex min-h-0 flex-col"
					:class="
						embedded
							? 'w-full flex-1'
							: 'w-[38%] min-w-[9.5rem] max-w-[15rem] shrink-0 border-0 border-r border-solid border-surface-5'
					"
				>
					<div class="flex gap-1 px-0.5 py-1">
						<IconButton
							v-tooltip="formatMessage(messages.newDm)"
							type="quiet"
							:label="formatMessage(messages.newDm)"
							@click="toggleNewDmPanel"
						>
							<MessageIcon />
						</IconButton>
						<IconButton
							v-tooltip="formatMessage(messages.newGroup)"
							type="quiet"
							:label="formatMessage(messages.newGroup)"
							@click="toggleNewGroupPanel"
						>
							<PlusIcon />
						</IconButton>
					</div>

					<div v-if="showNewDm" class="flex max-h-40 flex-col gap-0 overflow-y-auto pb-2">
						<button
							v-for="member in community"
							:key="member.id"
							type="button"
							class="chat-row flex items-center gap-2 border-0 bg-transparent px-2 py-1.5 text-left text-sm text-primary hover:bg-button-bg cursor-pointer"
							@click="openDm(member.id)"
						>
							<Avatar :src="headForNick(member.minecraft_nick)" size="22px" circle />
							<span class="truncate">{{ member.minecraft_nick }}</span>
						</button>
					</div>

					<div v-if="showNewGroup" class="flex flex-col gap-2 px-1 pb-2">
						<input
							v-model="groupName"
							type="text"
							class="w-full rounded-md border border-solid border-surface-5 bg-button-bg px-2 py-1.5 text-sm text-primary"
							:placeholder="formatMessage(messages.groupName)"
						/>
						<p class="m-0 text-[11px] text-secondary">{{ formatMessage(messages.pickMembers) }}</p>
						<p class="m-0 text-[11px] text-secondary">
							{{ formatMessage(messages.groupMinMembers) }}
						</p>
						<div class="max-h-28 overflow-y-auto">
							<label
								v-for="member in community"
								:key="member.id"
								class="flex cursor-pointer items-center gap-2 px-1 py-1 text-sm text-primary hover:bg-button-bg"
							>
								<input
									type="checkbox"
									:checked="groupPick.includes(member.id)"
									@change="toggleGroupMember(member.id)"
								/>
								<Avatar :src="headForNick(member.minecraft_nick)" size="18px" circle />
								<span class="truncate">{{ member.minecraft_nick }}</span>
							</label>
						</div>
						<Button
							type="colored"
							color="brand"
							class="w-full"
							:disabled="!canCreateGroup"
							@click="createGroup"
						>
							<UsersIcon />
							{{ formatMessage(messages.createGroup) }}
						</Button>
					</div>

					<p v-if="channels.length === 0" class="m-0 px-2 py-2 text-xs text-secondary">
						{{ formatMessage(messages.emptyChannels) }}
					</p>
					<div class="min-h-0 flex-1 overflow-y-auto pb-1">
						<button
							v-for="channel in channels"
							:key="channel.id"
							type="button"
							class="chat-row flex w-full items-center gap-2 border-0 px-2 py-2 text-left cursor-pointer"
							:class="
								selectedId === channel.id
									? 'chat-row--active text-contrast'
									: 'bg-transparent text-primary hover:bg-button-bg'
							"
							@click="selectedId = channel.id"
						>
							<Avatar
								:src="
									channel.kind === 'dm'
										? headForNick(titleFor(channel))
										: OCTRA_COMMUNITY_STEVE_HEAD
								"
								size="28px"
								circle
							/>
							<div class="min-w-0 flex-1">
								<span class="block truncate text-sm font-medium">{{ titleFor(channel) }}</span>
								<span class="block truncate text-[11px] text-secondary">
									{{ channelPreview(channel) }}
								</span>
							</div>
							<span
								v-if="(channel.unread_count ?? 0) > 0"
								class="shrink-0 rounded-full bg-brand px-1.5 py-0.5 text-[10px] font-semibold leading-none text-[var(--color-accent-contrast)]"
								:title="formatMessage(messages.unreadBadge, { count: channel.unread_count ?? 0 })"
							>
								{{ channel.unread_count }}
							</span>
						</button>
					</div>
				</div>

				<div
					v-show="showThreadPane"
					class="flex min-h-0 min-w-0 flex-1 flex-col"
					:class="
						embedded
							? 'bg-transparent'
							: 'bg-[color-mix(in_srgb,var(--color-bg)_70%,transparent)]'
					"
				>
					<div
						class="flex items-center justify-between gap-2 border-0 border-b border-solid border-surface-5 px-1 py-2"
					>
						<div class="flex min-w-0 items-center gap-1">
							<IconButton
								v-if="embedded"
								type="quiet"
								:label="formatMessage(messages.backToList)"
								@click="backToList"
							>
								<ChevronLeftIcon />
							</IconButton>
							<span class="min-w-0 truncate text-sm font-semibold text-contrast">
								{{ selected ? titleFor(selected) : formatMessage(messages.selectConversation) }}
							</span>
						</div>
						<div class="flex shrink-0 items-center gap-0.5">
							<IconButton
								v-if="canInvite"
								v-tooltip="formatMessage(messages.inviteMembers)"
								type="quiet"
								:label="formatMessage(messages.inviteMembers)"
								@click="toggleInvitePanel"
							>
								<UsersIcon />
							</IconButton>
							<IconButton
								v-if="canInvite"
								v-tooltip="formatMessage(messages.deleteGroup)"
								type="quiet"
								:label="formatMessage(messages.deleteGroup)"
								@click="toggleDeleteVotePanel"
							>
								<TrashIcon />
							</IconButton>
						</div>
					</div>

					<div
						v-if="showInvite && canInvite"
						class="flex flex-col gap-2 border-0 border-b border-solid border-surface-5 px-2 py-2"
					>
						<p class="m-0 text-[11px] text-secondary">{{ formatMessage(messages.pickMembers) }}</p>
						<div class="max-h-28 overflow-y-auto">
							<label
								v-for="member in inviteCandidates"
								:key="member.id"
								class="flex cursor-pointer items-center gap-2 px-1 py-1 text-sm text-primary hover:bg-button-bg"
							>
								<input
									type="checkbox"
									:checked="invitePick.includes(member.id)"
									@change="toggleInviteMember(member.id)"
								/>
								<Avatar :src="headForNick(member.minecraft_nick)" size="18px" circle />
								<span class="truncate">{{ member.minecraft_nick }}</span>
							</label>
						</div>
						<Button
							type="colored"
							color="brand"
							class="w-full"
							:disabled="invitePick.length === 0"
							@click="inviteMembers"
						>
							{{ formatMessage(messages.inviteConfirm) }}
						</Button>
					</div>

					<div
						v-if="showDeleteVote && canInvite"
						class="flex flex-col gap-2 border-0 border-b border-solid border-surface-5 px-2 py-2"
					>
						<p class="m-0 text-sm font-medium text-contrast">
							{{ formatMessage(messages.deleteVoteTitle) }}
						</p>
						<p class="m-0 text-[11px] text-secondary">
							{{ formatMessage(messages.deleteVoteConfirm) }}
						</p>
						<p v-if="deleteVote" class="m-0 text-[11px] text-secondary">
							{{
								formatMessage(messages.deleteVoteHint, {
									yes: deleteVote.yes_count,
									needed: deleteVote.needed,
									members: deleteVote.member_count,
								})
							}}
						</p>
						<div class="flex flex-wrap gap-2">
							<Button
								type="colored"
								color="brand"
								:disabled="deleteVoteBusy || deleteVote?.my_vote === true"
								@click="castDeleteVote(true)"
							>
								{{
									deleteVote?.active
										? formatMessage(messages.deleteVoteYes)
										: formatMessage(messages.deleteVoteStart)
								}}
							</Button>
							<Button
								v-if="deleteVote?.active"
								:disabled="deleteVoteBusy || deleteVote?.my_vote === false"
								@click="castDeleteVote(false)"
							>
								{{ formatMessage(messages.deleteVoteNo) }}
							</Button>
						</div>
					</div>

					<div
						v-if="mrpackPreview"
						class="mx-2 mt-2 rounded-md border border-solid border-surface-5 bg-button-bg p-3"
					>
						<p v-if="mrpackPreview.loading" class="m-0 text-sm text-secondary">
							{{ formatMessage(messages.installPreviewLoading) }}
						</p>
						<template v-else-if="mrpackPreview.preview">
							<p class="m-0 text-sm font-medium text-contrast">
								{{
									formatMessage(messages.installPreviewTitle, {
										name: mrpackPreview.preview.name || 'modpack',
									})
								}}
							</p>
							<p class="mt-1 mb-0 text-xs text-secondary">
								{{
									formatMessage(messages.installPreviewMeta, {
										gameVersion: mrpackPreview.preview.gameVersion || '—',
										modloader: mrpackPreview.preview.modloader || '—',
									})
								}}
							</p>
							<div class="mt-2 flex gap-2">
								<Button
									type="colored"
									color="brand"
									:disabled="!!installingUrl"
									@click="confirmMrpackInstall"
								>
									<DownloadIcon />
									{{ formatMessage(messages.installConfirm) }}
								</Button>
								<Button @click="cancelMrpackInstall">
									{{ formatMessage(commonMessages.cancelButton) }}
								</Button>
							</div>
						</template>
					</div>

					<div ref="scrollEl" class="min-h-0 flex-1 overflow-y-auto px-2 py-2">
						<p
							v-if="selected && messagesList.length === 0"
							class="m-auto max-w-[16rem] py-8 text-center text-sm text-secondary"
						>
							{{ formatMessage(messages.emptyMessages) }}
						</p>
						<p
							v-else-if="!selected && !embedded"
							class="m-auto max-w-[16rem] py-8 text-center text-sm text-secondary"
						>
							{{ formatMessage(messages.selectConversation) }}
						</p>
						<div
							v-for="row in bubbles"
							:key="row.message.id"
							class="group flex gap-2"
							:class="[row.mine ? 'flex-row-reverse' : 'flex-row', row.clustered ? 'mt-1' : 'mt-3']"
							@mouseenter="hoveredMessageId = row.message.id"
							@mouseleave="hoveredMessageId = null"
						>
							<div class="flex w-7 shrink-0 items-end justify-center self-end">
								<Avatar
									v-if="row.showAvatar"
									:src="headForNick(row.message.minecraft_nick)"
									:alt="row.message.minecraft_nick"
									size="24px"
									circle
								/>
							</div>
							<div
								class="relative flex max-w-[85%] flex-col"
								:class="row.mine ? 'items-end' : 'items-start'"
							>
								<span v-if="row.showName" class="mb-1 px-1 text-[11px] font-medium text-secondary">
									{{ row.message.minecraft_nick }}
								</span>
								<div
									v-tooltip="formatTime(row.message.created_at)"
									class="chat-bubble px-2.5 py-1.5 text-[13px] leading-snug whitespace-pre-wrap break-words"
									:class="[
										row.mine ? 'chat-bubble--mine' : 'chat-bubble--theirs',
										row.showAvatar ? 'chat-bubble--tailed' : 'chat-bubble--cluster',
									]"
									@contextmenu.prevent.stop="openMessageContextMenu(row, $event)"
								>
									<span
										v-if="row.message.pinned"
										class="mb-1 flex items-center gap-1 text-[10px] font-medium opacity-80"
									>
										<PinIcon class="size-3" />
										{{ formatMessage(messages.pinnedBadge) }}
									</span>
									<template v-if="row.message.deleted">
										<span class="italic opacity-70">{{
											formatMessage(messages.messageDeleted)
										}}</span>
									</template>
									<template v-else>
										<img
											v-if="row.message.attachment_url"
											:src="row.message.attachment_url"
											alt=""
											class="chat-attachment"
											loading="lazy"
											@click.stop="openAttachment(row.message.attachment_url)"
										/>
										<span v-if="row.message.body">{{ row.message.body }}</span>
										<div
											v-if="extractMrpackUrls(row.message.body).length > 0"
											class="mt-2 flex flex-col gap-1"
										>
											<button
												v-for="url in extractMrpackUrls(row.message.body)"
												:key="url"
												type="button"
												class="chat-pack-btn"
												:disabled="installingUrl === url || mrpackPreview?.loading"
												@click="beginMrpackInstall(url)"
											>
												<DownloadIcon class="size-3.5" />
												{{ formatMessage(messages.installPack) }}
											</button>
										</div>
									</template>
									<div v-if="row.message.reactions?.length" class="mt-1.5 flex flex-wrap gap-1">
										<button
											v-for="reaction in row.message.reactions"
											:key="reaction.emoji"
											type="button"
											class="chat-reaction"
											:class="{ 'chat-reaction--mine': reactionIsMine(reaction) }"
											@click="reactToMessage(row.message, reaction.emoji)"
										>
											{{ reaction.emoji }}
											<span class="chat-reaction__count">{{ reaction.count }}</span>
										</button>
									</div>
								</div>
								<div
									v-if="!row.message.deleted && hoveredMessageId === row.message.id"
									class="absolute -top-3 z-10 flex items-center gap-0.5 rounded-md border border-solid border-surface-5 bg-bg-raised px-1 py-0.5"
									:class="row.mine ? 'right-0' : 'left-0'"
								>
									<button
										v-for="emoji in REACTION_EMOJIS"
										:key="emoji"
										type="button"
										class="chat-reaction-pick border-0 bg-transparent px-0.5 text-sm leading-none cursor-pointer hover:scale-110"
										:class="{
											'chat-reaction-pick--mine': row.message.reactions?.some(
												(r) => r.emoji === emoji && reactionIsMine(r),
											),
										}"
										@click="reactToMessage(row.message, emoji)"
									>
										{{ emoji }}
									</button>
									<button
										type="button"
										class="inline-flex size-5 items-center justify-center rounded border-0 bg-transparent text-secondary cursor-pointer hover:bg-button-bg hover:text-contrast"
										:title="formatMessage(row.message.pinned ? messages.unpin : messages.pin)"
										@click="togglePin(row.message)"
									>
										<PinIcon class="size-3" />
									</button>
									<button
										v-if="row.mine"
										type="button"
										class="inline-flex size-5 items-center justify-center rounded border-0 bg-transparent text-secondary cursor-pointer hover:bg-button-bg hover:text-contrast"
										:title="formatMessage(messages.delete)"
										@click="deleteMessage(row.message)"
									>
										<TrashIcon class="size-3" />
									</button>
								</div>
							</div>
						</div>
					</div>
					<div
						v-if="selected"
						class="flex shrink-0 items-center gap-2 border-0 border-t border-solid border-surface-5 p-2"
					>
						<input
							v-model="draft"
							type="text"
							class="min-h-9 w-full rounded-md border border-solid border-surface-5 bg-button-bg px-3 py-2 text-sm text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.placeholder)"
							:disabled="sending"
							@keydown="onKeydown"
						/>
						<button
							type="button"
							class="flex size-9 shrink-0 items-center justify-center rounded-md border-0 bg-brand text-[var(--color-accent-contrast)] cursor-pointer disabled:opacity-40"
							:aria-label="formatMessage(messages.send)"
							:disabled="sending || !draft.trim()"
							@click="send"
						>
							<SendIcon class="size-4" />
						</button>
					</div>
				</div>
			</div>
		</template>
	</component>
</template>

<style scoped lang="scss">
.octra-chat-rail {
	animation: chat-rail-in 0.28s cubic-bezier(0.32, 0.72, 0, 1) both;
}

.octra-chat-rail__accent {
	position: absolute;
	top: 0;
	bottom: 0;
	left: 0;
	width: 0.2rem;
	background: var(--color-brand);
	z-index: 1;
}

.octra-chat-rail__header {
	background: var(--surface-2);
}

.octra-chat-embedded {
	display: flex;
	flex: 1 1 auto;
	flex-direction: column;
	height: 100%;
	min-height: 0;
}

.chat-row + .chat-row {
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 70%, transparent);
}

.chat-row--active {
	background: color-mix(in srgb, var(--color-brand) 12%, transparent);
	box-shadow: inset 2px 0 0 var(--color-brand);
}

.chat-bubble {
	position: relative;
}

.chat-attachment {
	display: block;
	max-width: min(100%, 18rem);
	max-height: 14rem;
	width: auto;
	height: auto;
	border-radius: var(--radius-sm);
	margin-bottom: 0.35rem;
	cursor: zoom-in;
	object-fit: contain;
	background: color-mix(in srgb, var(--surface-1) 60%, transparent);
}

.chat-bubble--mine {
	background: color-mix(in srgb, var(--color-brand) 22%, var(--surface-3));
	border: 1px solid color-mix(in srgb, var(--color-brand) 35%, transparent);
	color: var(--color-contrast);
}

.chat-bubble--theirs {
	background: var(--surface-3);
	border: 1px solid color-mix(in srgb, var(--surface-5) 90%, transparent);
	color: var(--color-primary);
}

.chat-bubble--cluster.chat-bubble--mine,
.chat-bubble--cluster.chat-bubble--theirs {
	border-radius: var(--radius-md);
}

.chat-bubble--tailed.chat-bubble--theirs {
	border-radius: var(--radius-md) var(--radius-md) var(--radius-md) 0.2rem;

	&::before {
		border-color: transparent var(--surface-3) transparent transparent;
		border-style: solid;
		border-width: 6px 7px 6px 0;
		bottom: 0.55rem;
		content: '';
		left: -7px;
		position: absolute;
	}
}

.chat-bubble--tailed.chat-bubble--mine {
	border-radius: var(--radius-md) var(--radius-md) 0.2rem var(--radius-md);

	&::before {
		border-color: transparent transparent transparent
			color-mix(in srgb, var(--color-brand) 22%, var(--surface-3));
		border-style: solid;
		border-width: 6px 0 6px 7px;
		bottom: 0.55rem;
		content: '';
		position: absolute;
		right: -7px;
	}
}

.chat-pack-btn {
	align-items: center;
	background: color-mix(in srgb, var(--color-brand) 14%, transparent);
	border: 1px solid color-mix(in srgb, var(--color-brand) 28%, transparent);
	border-radius: var(--radius-md);
	color: var(--color-primary);
	cursor: pointer;
	display: inline-flex;
	font-size: 0.75rem;
	font-weight: 600;
	gap: 0.35rem;
	padding: 0.3rem 0.5rem;

	&:hover {
		background: color-mix(in srgb, var(--color-brand) 22%, transparent);
	}

	&:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}
}

.chat-reaction {
	align-items: center;
	background: color-mix(in srgb, var(--surface-5) 55%, transparent);
	border: 1px solid color-mix(in srgb, var(--surface-5) 90%, transparent);
	border-radius: 999px;
	color: var(--color-primary);
	cursor: pointer;
	display: inline-flex;
	font-size: 0.6875rem;
	gap: 0.25rem;
	line-height: 1;
	padding: 0.2rem 0.45rem;
	transition: background 0.12s ease, border-color 0.12s ease;

	&:hover {
		background: color-mix(in srgb, var(--color-brand) 12%, transparent);
		border-color: color-mix(in srgb, var(--color-brand) 35%, transparent);
	}
}

.chat-reaction--mine {
	background: color-mix(in srgb, var(--color-brand) 18%, transparent);
	border-color: color-mix(in srgb, var(--color-brand) 55%, transparent);
	color: var(--color-contrast);
}

.chat-reaction__count {
	opacity: 0.85;
}

.chat-reaction-pick--mine {
	border-radius: 0.25rem;
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-brand) 65%, transparent);
	background: color-mix(in srgb, var(--color-brand) 16%, transparent);
}

@keyframes chat-rail-in {
	from {
		opacity: 0;
		transform: translateX(-0.5rem);
	}
	to {
		opacity: 1;
		transform: translateX(0);
	}
}

@media (prefers-reduced-motion: reduce) {
	.octra-chat-rail {
		animation: none;
	}
}
</style>
