<script setup lang="ts">
import {
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
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

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
	octraChatChannels,
	octraChatCreateGroup,
	octraChatDeleteMessage,
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
}>()

const emit = defineEmits<{
	close: []
	signIn: []
	unreadChanged: [total: number]
}>()

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
const groupName = ref('')
const groupPick = ref<number[]>([])
const invitePick = ref<number[]>([])
const hoveredMessageId = ref<number | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

const SKINS_FALLBACK_ORIGIN = 'http://92.5.186.6'

const selected = computed(() => channels.value.find((c) => c.id === selectedId.value) ?? null)

const unreadTotal = computed(() =>
	channels.value.reduce((sum, channel) => sum + (channel.unread_count ?? 0), 0),
)

const canInvite = computed(
	() => !!selected.value && selected.value.kind === 'group' && selected.value.name !== 'Everyone',
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
			showAvatar: !mine && !sameAsNext,
			clustered: sameAsPrev,
		}
	})
})

function titleFor(channel: ChatChannel) {
	return channelTitle(channel, props.session?.minecraft_nick)
}

function formatTime(value: string) {
	return new Date(value).toLocaleTimeString([], {
		hour: '2-digit',
		minute: '2-digit',
	})
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
	channels.value = await octraChatChannels()
	if (selectedId.value == null && channels.value.length > 0) {
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
	}, 8_000)
}

watch(
	() => [props.open, props.session?.username ?? null] as const,
	async ([isOpen]) => {
		stopPolling()
		if (!isOpen || !props.session) {
			messagesList.value = []
			return
		}
		try {
			await Promise.all([refreshChannels(), refreshCommunity()])
			await loadMessages(true)
			startPolling()
		} catch (error) {
			handleSevereError(error, handleError)
		}
	},
	{ immediate: true },
)

watch(selectedId, async () => {
	if (!props.open || !props.session || selectedId.value == null) return
	showInvite.value = false
	invitePick.value = []
	mrpackPreview.value = null
	try {
		await loadMessages(true)
	} catch (error) {
		handleSevereError(error, handleError)
	}
})

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
	if (!name) return
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

function toggleNewDmPanel() {
	showNewDm.value = !showNewDm.value
	showNewGroup.value = false
}

function toggleNewGroupPanel() {
	showNewGroup.value = !showNewGroup.value
	showNewDm.value = false
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
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function reactToMessage(message: ChatMessage, emoji: string) {
	try {
		await octraChatReactMessage(message.id, emoji)
		await loadMessages(true)
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
	unreadBadge: {
		id: 'octra.chat.unread',
		defaultMessage: '{count} unread',
	},
})
</script>

<template>
	<aside
		v-if="open"
		class="octra-chat-rail relative flex h-full min-h-0 w-[min(560px,52vw)] shrink-0 flex-col border-0 border-r border-solid border-surface-5 bg-bg-raised"
	>
		<div class="octra-chat-rail__accent" aria-hidden="true" />
		<div
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
			<div class="flex flex-col gap-2 p-3">
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.signInHint) }}</p>
				<Button type="colored" color="brand" @click="emit('signIn')">
					{{ formatMessage(messages.signInAction) }}
				</Button>
			</div>
		</template>

		<template v-else>
			<div class="flex min-h-0 flex-1">
				<div
					class="flex w-[38%] min-w-[9.5rem] max-w-[15rem] shrink-0 flex-col border-0 border-r border-solid border-surface-5"
				>
					<div class="flex gap-1 p-2">
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

					<div v-if="showNewDm" class="flex max-h-40 flex-col gap-0.5 overflow-y-auto px-2 pb-2">
						<button
							v-for="member in community"
							:key="member.id"
							type="button"
							class="flex items-center gap-2 rounded-lg border-0 bg-transparent px-2 py-1.5 text-left text-sm text-primary hover:bg-button-bg cursor-pointer"
							@click="openDm(member.id)"
						>
							<Avatar :src="headForNick(member.minecraft_nick)" size="22px" circle />
							<span class="truncate">{{ member.minecraft_nick }}</span>
						</button>
					</div>

					<div v-if="showNewGroup" class="flex flex-col gap-2 px-2 pb-2">
						<input
							v-model="groupName"
							type="text"
							class="w-full rounded-lg border border-solid border-surface-5 bg-button-bg px-2 py-1.5 text-sm text-primary"
							:placeholder="formatMessage(messages.groupName)"
						/>
						<p class="m-0 text-[11px] text-secondary">{{ formatMessage(messages.pickMembers) }}</p>
						<div class="max-h-28 overflow-y-auto">
							<label
								v-for="member in community"
								:key="member.id"
								class="flex cursor-pointer items-center gap-2 rounded-lg px-1 py-1 text-sm text-primary hover:bg-button-bg"
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
							:disabled="!groupName.trim()"
							@click="createGroup"
						>
							<UsersIcon />
							{{ formatMessage(messages.createGroup) }}
						</Button>
					</div>

					<p v-if="channels.length === 0" class="m-0 px-3 py-2 text-xs text-secondary">
						{{ formatMessage(messages.emptyChannels) }}
					</p>
					<div class="min-h-0 flex-1 overflow-y-auto px-1 pb-2">
						<button
							v-for="channel in channels"
							:key="channel.id"
							type="button"
							class="mb-0.5 flex w-full items-center gap-2 rounded-xl border-0 px-2 py-1.5 text-left cursor-pointer"
							:class="
								selectedId === channel.id
									? 'bg-button-bg text-contrast'
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
									{{ channel.last_body || '—' }}
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
					class="flex min-w-0 flex-1 flex-col bg-[color-mix(in_srgb,var(--color-bg)_70%,transparent)]"
				>
					<div
						class="flex items-center justify-between gap-2 border-0 border-b border-solid border-surface-5 px-3 py-2.5"
					>
						<span class="min-w-0 truncate text-sm font-semibold text-contrast">
							{{ selected ? titleFor(selected) : formatMessage(messages.selectConversation) }}
						</span>
						<IconButton
							v-if="canInvite"
							v-tooltip="formatMessage(messages.inviteMembers)"
							type="quiet"
							:label="formatMessage(messages.inviteMembers)"
							@click="showInvite = !showInvite"
						>
							<UsersIcon />
						</IconButton>
					</div>

					<div
						v-if="showInvite && canInvite"
						class="flex flex-col gap-2 border-0 border-b border-solid border-surface-5 px-3 py-2"
					>
						<p class="m-0 text-[11px] text-secondary">{{ formatMessage(messages.pickMembers) }}</p>
						<div class="max-h-28 overflow-y-auto">
							<label
								v-for="member in inviteCandidates"
								:key="member.id"
								class="flex cursor-pointer items-center gap-2 rounded-lg px-1 py-1 text-sm text-primary hover:bg-button-bg"
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
						v-if="mrpackPreview"
						class="mx-3 mt-2 rounded-xl border border-solid border-surface-5 bg-button-bg p-3"
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

					<div ref="scrollEl" class="flex min-h-0 flex-1 flex-col overflow-y-auto px-3 py-3">
						<p
							v-if="selected && messagesList.length === 0"
							class="m-auto max-w-[16rem] text-center text-sm text-secondary"
						>
							{{ formatMessage(messages.emptyMessages) }}
						</p>
						<div
							v-for="row in bubbles"
							:key="row.message.id"
							class="group flex gap-2"
							:class="[row.mine ? 'flex-row-reverse' : 'flex-row', row.clustered ? 'mt-1' : 'mt-3']"
							@mouseenter="hoveredMessageId = row.message.id"
							@mouseleave="hoveredMessageId = null"
						>
							<div class="flex w-8 shrink-0 items-end justify-center">
								<Avatar
									v-if="row.showAvatar"
									:src="headForNick(row.message.minecraft_nick)"
									:alt="row.message.minecraft_nick"
									size="28px"
									circle
								/>
							</div>
							<div
								class="relative flex max-w-[78%] flex-col"
								:class="row.mine ? 'items-end' : 'items-start'"
							>
								<span v-if="row.showName" class="mb-1 px-1 text-[11px] font-medium text-secondary">
									{{ row.message.minecraft_nick }}
								</span>
								<div
									class="px-3 py-2 text-sm leading-snug whitespace-pre-wrap break-words shadow-sm"
									:class="
										row.mine
											? 'rounded-2xl rounded-br-md bg-brand text-[var(--color-accent-contrast)]'
											: 'rounded-2xl rounded-bl-md bg-button-bg text-primary'
									"
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
										{{ row.message.body }}
										<div
											v-if="extractMrpackUrls(row.message.body).length > 0"
											class="mt-2 flex flex-col gap-1"
										>
											<button
												v-for="url in extractMrpackUrls(row.message.body)"
												:key="url"
												type="button"
												class="inline-flex items-center gap-1.5 rounded-lg border-0 px-2 py-1 text-xs font-medium cursor-pointer"
												:class="
													row.mine
														? 'bg-black/20 text-[var(--color-accent-contrast)] hover:bg-black/30'
														: 'bg-surface-3 text-primary hover:bg-surface-4'
												"
												:disabled="installingUrl === url || mrpackPreview?.loading"
												@click="beginMrpackInstall(url)"
											>
												<DownloadIcon class="size-3.5" />
												{{ formatMessage(messages.installPack) }}
											</button>
										</div>
									</template>
									<div v-if="row.message.reactions?.length" class="mt-1.5 flex flex-wrap gap-1">
										<span
											v-for="reaction in row.message.reactions"
											:key="reaction.emoji"
											class="inline-flex items-center gap-0.5 rounded-full bg-black/15 px-1.5 py-0.5 text-[11px]"
										>
											{{ reaction.emoji }}
											<span class="opacity-80">{{ reaction.count }}</span>
										</span>
									</div>
								</div>
								<div
									v-if="!row.message.deleted && hoveredMessageId === row.message.id"
									class="absolute -top-3 z-10 flex items-center gap-0.5 rounded-lg border border-solid border-surface-5 bg-bg-raised px-1 py-0.5 shadow-sm"
									:class="row.mine ? 'right-0' : 'left-0'"
								>
									<button
										v-for="emoji in REACTION_EMOJIS"
										:key="emoji"
										type="button"
										class="border-0 bg-transparent px-0.5 text-sm leading-none cursor-pointer hover:scale-110"
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
								<span class="mt-0.5 px-1 text-[10px] text-secondary opacity-80">
									{{ formatTime(row.message.created_at) }}
								</span>
							</div>
						</div>
					</div>
					<div
						v-if="selected"
						class="flex items-center gap-2 border-0 border-t border-solid border-surface-5 p-2.5"
					>
						<input
							v-model="draft"
							type="text"
							class="min-h-10 w-full rounded-full border border-solid border-surface-5 bg-button-bg px-4 py-2 text-sm text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.placeholder)"
							:disabled="sending"
							@keydown="onKeydown"
						/>
						<button
							type="button"
							class="flex size-10 shrink-0 items-center justify-center rounded-full border-0 bg-brand text-[var(--color-accent-contrast)] cursor-pointer disabled:opacity-40"
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
	</aside>
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
	background: linear-gradient(
		180deg,
		var(--color-brand) 0%,
		color-mix(in srgb, var(--color-brand) 35%, transparent) 100%
	);
	z-index: 1;
}

.octra-chat-rail__header {
	background: linear-gradient(
		90deg,
		color-mix(in srgb, var(--color-brand) 16%, transparent) 0%,
		transparent 72%
	);
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
