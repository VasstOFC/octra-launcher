<script setup lang="ts">
import {
	MessageIcon,
	PackageSearchIcon,
	PlayIcon,
	SearchIcon,
} from '@modrinth/assets'
import type { ButtonMenuOption } from '@modrinth/ui'
import {
	Avatar,
	Button,
	ContextMenu,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import PlayWithFriendModal from '@/components/ui/friends/PlayWithFriendModal.vue'
import OctraChatPanel from '@/components/ui/friends/OctraChatPanel.vue'
import {
	canPlayWithFriend,
	canViewFriendPack,
} from '@/composables/play-with-friend'
import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { octraCommunity } from '@/helpers/octra-account.js'

type OctraAccountSession = {
	token: string
	username: string
	minecraft_nick: string
	profile_uuid: string
	account_type?: string
}

type OctraCommunityMember = {
	id: number
	minecraft_nick: string
	profile_uuid: string
	account_type: string
	created_at: string
	avatar_url: string
	presence?: string
	instance_name?: string | null
	join_address?: string | null
	pack_project_id?: string | null
	pack_version_id?: string | null
	last_seen?: string | null
}

const props = defineProps<{
	session: OctraAccountSession | null
	loadingSession?: boolean
	/** True while the friends sidebar is visible. */
	panelActive?: boolean
	unreadTotal?: number
}>()

const emit = defineEmits<{
	signIn: []
	register: []
	unreadChanged: [total: number]
	chatActive: [active: boolean]
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime({ numeric: 'auto', style: 'short' })
const search = ref('')
const panelTab = ref<'friends' | 'chat'>('friends')
const chatUnread = ref(0)
const joiningId = ref<number | null>(null)
const previousPresence = ref<Map<number, string>>(new Map())
const presenceReady = ref(false)
const memberOptions = useTemplateRef('memberOptions')
const playWithModal = useTemplateRef<InstanceType<typeof PlayWithFriendModal>>('playWithModal')
const chatPanel = useTemplateRef<InstanceType<typeof OctraChatPanel>>('chatPanel')

const badgeUnread = computed(() =>
	Math.max(props.unreadTotal ?? 0, chatUnread.value),
)

/** Strip markup so API nick/status never render as HTML (text-only sidebar). */
function plainText(value: string | null | undefined): string {
	if (value == null) return ''
	return String(value).replace(/<[^>]*>/g, '')
}

const query = useQuery({
	queryKey: computed(() => ['octra-community', props.session?.username ?? null]),
	queryFn: () => octraCommunity(),
	enabled: computed(() => !!props.session),
	staleTime: 10_000,
	refetchInterval: 15_000,
})

const snapshot = computed(() => query.data.value)
const connected = computed(() => !!props.session && !!snapshot.value?.connected)
const members = computed<OctraCommunityMember[]>(() => {
	const raw = snapshot.value?.members ?? []
	return raw.map((member) => ({
		...member,
		minecraft_nick: plainText(member.minecraft_nick),
		instance_name:
			member.instance_name != null ? plainText(member.instance_name) : member.instance_name,
	}))
})
const listLoading = computed(
	() => !!props.loadingSession || (!!props.session && query.isLoading.value),
)

const filtered = computed(() => {
	const q = plainText(search.value).trim().toLowerCase()
	const listMembers = members.value.slice().sort((a, b) => {
		const rank = (presence?: string) => {
			if (presence === 'ingame') return 0
			if (presence === 'launcher') return 1
			return 2
		}
		const byPresence = rank(a.presence) - rank(b.presence)
		if (byPresence !== 0) return byPresence
		return a.minecraft_nick.localeCompare(b.minecraft_nick, undefined, { sensitivity: 'base' })
	})
	if (!q) return listMembers
	return listMembers.filter((member) => member.minecraft_nick.toLowerCase().includes(q))
})

const { avatarFor } = useOctraCommunityAvatars(members)

watch(
	members,
	(nextMembers) => {
		const nextMap = new Map<number, string>()
		for (const member of nextMembers) {
			nextMap.set(member.id, member.presence ?? 'offline')
		}
		if (presenceReady.value) {
			for (const member of nextMembers) {
				const prev = previousPresence.value.get(member.id)
				const next = member.presence ?? 'offline'
				if (prev && prev !== 'ingame' && next === 'ingame') {
					addNotification({
						title: formatMessage(messages.friendInGame, { nick: member.minecraft_nick }),
						text: '',
						type: 'success',
					})
				}
			}
		}
		previousPresence.value = nextMap
		presenceReady.value = true
	},
	{ deep: true },
)

watch(panelTab, (tab) => {
	emit('chatActive', tab === 'chat')
})

watch(
	() => props.session?.username ?? null,
	() => {
		previousPresence.value = new Map()
		presenceReady.value = false
		panelTab.value = 'friends'
		emit('chatActive', false)
	},
)

function presenceDotClass(presence?: string) {
	if (presence === 'ingame' || presence === 'launcher') return 'bg-brand-green'
	return 'bg-secondary opacity-40'
}

function presenceLabel(member: OctraCommunityMember) {
	if (member.presence === 'ingame') {
		if (member.instance_name) {
			return formatMessage(messages.inGame, { name: member.instance_name })
		}
		return formatMessage(messages.inGameUnknown)
	}
	if (member.presence === 'launcher') {
		return formatMessage(messages.inLauncher)
	}
	if (member.last_seen) {
		const relative = formatRelativeTime(member.last_seen)
		if (relative) {
			return formatMessage(messages.lastSeen, { time: relative })
		}
	}
	return formatMessage(messages.offline)
}

function createContextMenuOptions(member: OctraCommunityMember): ButtonMenuOption[] {
	const options: ButtonMenuOption[] = [
		{
			id: 'message-player',
			label: formatMessage(messages.messagePlayer),
			icon: MessageIcon,
			action: () => void openChatDm(member.id),
		},
	]
	if (canJoin(member)) {
		options.unshift({
			id: 'play-with',
			label: formatMessage(messages.playWith),
			icon: PlayIcon,
			action: () => void openPlayWith(member),
		})
	}
	if (canViewPack(member)) {
		options.push({
			id: 'view-pack',
			label: formatMessage(messages.viewPack),
			icon: PackageSearchIcon,
			action: () => void viewFriendPack(member),
		})
	}
	return options
}

function openMemberContextMenu(event: MouseEvent, member: OctraCommunityMember) {
	memberOptions.value?.open(event, createContextMenuOptions(member))
}

function canJoin(member: OctraCommunityMember) {
	return canPlayWithFriend(member)
}

function canViewPack(member: OctraCommunityMember) {
	return canViewFriendPack(member)
}

async function viewFriendPack(member: OctraCommunityMember) {
	await playWithModal.value?.viewPack(member)
}

async function openPlayWith(member: OctraCommunityMember) {
	joiningId.value = member.id
	await playWithModal.value?.open(member)
}

function onPlayWithClosed() {
	joiningId.value = null
}

async function joinFriend(member: OctraCommunityMember) {
	await openPlayWith(member)
}

async function openChatDm(userId: number) {
	panelTab.value = 'chat'
	emit('chatActive', true)
	await nextTick()
	await chatPanel.value?.openDm?.(userId)
}

function setTab(tab: 'friends' | 'chat') {
	panelTab.value = tab
	emit('chatActive', tab === 'chat')
}

function isChatActive() {
	return panelTab.value === 'chat'
}

function onChatUnread(total: number) {
	chatUnread.value = total
	emit('unreadChanged', total)
}

defineExpose({
	setTab,
	isChatActive,
	openChatDm,
})

const messages = defineMessages({
	heading: {
		id: 'octra.community.heading',
		defaultMessage: 'Friends',
	},
	tabFriends: {
		id: 'octra.community.tab.friends',
		defaultMessage: 'Friends',
	},
	tabChat: {
		id: 'octra.community.tab.chat',
		defaultMessage: 'Chat',
	},
	search: {
		id: 'octra.community.search',
		defaultMessage: 'Search friends...',
	},
	signIn: {
		id: 'octra.community.sign-in',
		defaultMessage: 'Sign in to Octra to see everyone who plays with this launcher.',
	},
	signInAction: {
		id: 'octra.community.sign-in-action',
		defaultMessage: 'Log in',
	},
	registerAction: {
		id: 'octra.community.register-action',
		defaultMessage: 'Connect account',
	},
	empty: {
		id: 'octra.community.empty',
		defaultMessage: "You don't have any friends yet :(",
	},
	noMatch: {
		id: 'octra.community.no-match',
		defaultMessage: `No friends matching ''{query}''`,
	},
	offline: {
		id: 'octra.community.offline',
		defaultMessage: 'Offline',
	},
	count: {
		id: 'octra.community.count',
		defaultMessage: '{count, plural, one {# player} other {# players}}',
	},
	connected: {
		id: 'octra.community.connected',
		defaultMessage: 'Connected',
	},
	disconnected: {
		id: 'octra.community.disconnected',
		defaultMessage: 'Disconnected',
	},
	inLauncher: {
		id: 'octra.community.in-launcher',
		defaultMessage: 'In the launcher',
	},
	inGame: {
		id: 'octra.community.in-game',
		defaultMessage: 'Playing: {name}',
	},
	inGameUnknown: {
		id: 'octra.community.in-game-unknown',
		defaultMessage: 'In game',
	},
	lastSeen: {
		id: 'octra.community.last-seen',
		defaultMessage: 'Last seen {time}',
	},
	join: {
		id: 'octra.community.join',
		defaultMessage: 'Join',
	},
	playWith: {
		id: 'octra.community.play-with',
		defaultMessage: 'Play with',
	},
	friendInGame: {
		id: 'octra.community.friend-in-game',
		defaultMessage: '{nick} is in game',
	},
	messagePlayer: {
		id: 'octra.community.message-player',
		defaultMessage: 'Message player',
	},
	viewPack: {
		id: 'octra.community.view-pack',
		defaultMessage: 'Zobacz paczkę',
	},
	memberActionsLabel: {
		id: 'octra.community.actions.label',
		defaultMessage: 'Friend actions',
	},
})
</script>

<template>
	<div class="community-panel flex h-full min-h-0 flex-1 flex-col gap-3">
		<ContextMenu ref="memberOptions" :label="formatMessage(messages.memberActionsLabel)" />
		<PlayWithFriendModal ref="playWithModal" @closed="onPlayWithClosed" />
		<div class="flex items-start justify-between gap-2">
			<div class="min-w-0">
				<h3 class="m-0 text-sm font-semibold uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.heading) }}
				</h3>
				<div class="mt-1 flex items-center gap-1.5 text-[11px] leading-none text-secondary">
					<span
						class="size-1.5 shrink-0 rounded-full"
						:class="connected ? 'bg-brand-green opacity-70' : 'bg-secondary opacity-40'"
					/>
					<span>
						{{
							connected ? formatMessage(messages.connected) : formatMessage(messages.disconnected)
						}}
					</span>
				</div>
			</div>
			<span v-if="session && members.length > 0" class="text-xs text-secondary">
				{{ formatMessage(messages.count, { count: members.length }) }}
			</span>
		</div>

		<template v-if="listLoading">
			<div v-for="n in 4" :key="n" class="flex gap-2 items-center animate-pulse">
				<div class="min-w-9 min-h-9 bg-button-bg rounded-full"></div>
				<div class="flex flex-col w-full">
					<div class="h-3 bg-button-bg rounded-full w-1/2 mb-1"></div>
					<div class="h-2.5 bg-button-bg rounded-full w-3/4"></div>
				</div>
			</div>
		</template>

		<template v-else-if="!session">
			<p class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.signIn) }}
			</p>
			<div class="flex flex-col gap-2">
				<Button type="colored" color="brand" class="w-full" @click="emit('signIn')">
					{{ formatMessage(messages.signInAction) }}
				</Button>
				<Button class="w-full" @click="emit('register')">
					{{ formatMessage(messages.registerAction) }}
				</Button>
			</div>
		</template>

		<template v-else>
			<div
				class="community-tabs grid shrink-0 grid-cols-2 gap-0 border-0 border-b border-solid border-surface-5"
				role="tablist"
				:aria-label="formatMessage(messages.heading)"
			>
				<button
					type="button"
					role="tab"
					class="community-tab"
					:class="{ 'community-tab--active': panelTab === 'friends' }"
					:aria-selected="panelTab === 'friends'"
					@click="panelTab = 'friends'"
				>
					{{ formatMessage(messages.tabFriends) }}
				</button>
				<button
					type="button"
					role="tab"
					class="community-tab relative"
					:class="{ 'community-tab--active': panelTab === 'chat' }"
					:aria-selected="panelTab === 'chat'"
					@click="panelTab = 'chat'"
				>
					{{ formatMessage(messages.tabChat) }}
					<span
						v-if="badgeUnread > 0 && panelTab !== 'chat'"
						class="ml-1 inline-flex min-w-[1rem] items-center justify-center rounded-full bg-brand px-1 text-[10px] font-semibold leading-4 text-[var(--color-accent-contrast)]"
					>
						{{ badgeUnread > 99 ? '99+' : badgeUnread }}
					</span>
				</button>
			</div>

			<template v-if="panelTab === 'friends'">
				<template v-if="members.length === 0">
					<p class="m-0 text-sm text-secondary">
						{{ formatMessage(messages.empty) }}
					</p>
				</template>

				<template v-else>
					<Input
						v-if="members.length > 5"
						v-model="search"
						:icon="SearchIcon"
						type="text"
						appearance="transparent"
						:placeholder="formatMessage(messages.search)"
						clearable
						input-class="!text-primary !placeholder:text-primary"
						wrapper-class="!border-button-bg [&>span:first-child]:!text-primary [&>span:first-child]:!opacity-100"
						@keyup.esc="search = ''"
					/>
					<div class="community-list flex flex-col">
						<div
							v-for="(member, index) in filtered"
							:key="member.id"
							class="community-row grid grid-cols-[auto_1fr_auto] items-center gap-2 px-1 py-2 select-none hover:bg-surface-3"
							:style="{ '--stagger': `${Math.min(index, 8) * 28}ms` }"
							@contextmenu.prevent.stop="(event) => openMemberContextMenu(event, member)"
						>
							<div class="relative shrink-0">
								<Avatar
									:src="avatarFor(member)"
									:alt="member.minecraft_nick"
									size="32px"
									circle
								/>
								<span
									class="absolute bottom-0 right-0 size-2 rounded-full ring-2 ring-[var(--color-raised-bg)]"
									:class="presenceDotClass(member.presence)"
								/>
							</div>
							<div class="flex min-w-0 flex-col">
								<span class="truncate text-sm text-contrast">{{ member.minecraft_nick }}</span>
								<span class="truncate text-xs text-secondary">
									{{ presenceLabel(member) }}
								</span>
							</div>
							<IconButton
								v-if="canJoin(member)"
								v-tooltip="formatMessage(messages.playWith)"
								type="standard"
								color="brand"
								:label="formatMessage(messages.playWith)"
								:disabled="joiningId === member.id"
								@click="joinFriend(member)"
							>
								<PlayIcon />
							</IconButton>
						</div>
					</div>
					<p v-if="filtered.length === 0 && search" class="m-0 text-sm text-secondary">
						{{ formatMessage(messages.noMatch, { query: plainText(search) }) }}
					</p>
				</template>
			</template>

			<div v-else class="community-chat flex min-h-0 flex-1 flex-col">
				<OctraChatPanel
					ref="chatPanel"
					class="min-h-0 flex-1"
					embedded
					:session="session"
					:open="!!panelActive && panelTab === 'chat'"
					@sign-in="emit('signIn')"
					@unread-changed="onChatUnread"
				/>
			</div>
		</template>
	</div>
</template>

<style scoped>
.community-tabs {
	margin: 0 -0.25rem;
}

.community-tab {
	background: transparent;
	border: 0;
	border-bottom: 2px solid transparent;
	color: var(--color-secondary);
	cursor: pointer;
	font-size: 0.75rem;
	font-weight: 600;
	margin-bottom: -1px;
	padding: 0.5rem 0.25rem;
	text-align: center;
}

.community-tab:hover {
	color: var(--color-primary);
}

.community-tab--active {
	border-bottom-color: var(--color-brand);
	color: var(--color-brand);
}

.community-row + .community-row {
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 70%, transparent);
}

.community-chat {
	min-height: 0;
}

@media (prefers-reduced-motion: no-preference) {
	:global(.app-sidebar.open) .community-row {
		animation: community-row-in 0.28s cubic-bezier(0.32, 0.72, 0, 1) both;
		animation-delay: var(--stagger, 0ms);
	}
}

@keyframes community-row-in {
	from {
		opacity: 0;
		transform: translateX(0.5rem);
	}
	to {
		opacity: 1;
		transform: translateX(0);
	}
}
</style>
