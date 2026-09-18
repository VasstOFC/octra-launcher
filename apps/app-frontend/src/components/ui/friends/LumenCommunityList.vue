<script setup lang="ts">
import { MessageIcon, PackageSearchIcon, PlayIcon, SearchIcon } from '@lumen/assets'
import type { ButtonMenuOption } from '@lumen/ui'
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
} from '@lumen/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import LumenChatPanel from '@/components/ui/friends/LumenChatPanel.vue'
import PlayWithFriendModal from '@/components/ui/friends/PlayWithFriendModal.vue'
import { canPlayWithFriend, canViewFriendPack } from '@/composables/play-with-friend'
import { useLumenCommunityAvatars } from '@/composables/use-lumen-community-avatars'
import { LumenCommunity } from '@/helpers/lumen-account.js'

type LumenAccountSession = {
	token: string
	username: string
	minecraft_nick: string
	profile_uuid: string
	account_type?: string
}

type LumenCommunityMember = {
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
	session: LumenAccountSession | null
	loadingSession?: boolean
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
const chatPanel = useTemplateRef<InstanceType<typeof LumenChatPanel>>('chatPanel')
const offlineExpanded = ref(false)

const badgeUnread = computed(() => Math.max(props.unreadTotal ?? 0, chatUnread.value))

function plainText(value: string | null | undefined): string {
	if (value == null) return ''
	return String(value).replace(/<[^>]*>/g, '')
}

const query = useQuery({
	queryKey: computed(() => ['Lumen-community', props.session?.username ?? null]),
	queryFn: () => LumenCommunity(),
	enabled: computed(() => !!props.session),
	staleTime: 10_000,
	refetchInterval: 15_000,
})

const snapshot = computed(() => query.data.value)
const connected = computed(() => !!props.session && !!snapshot.value?.connected)
const members = computed<LumenCommunityMember[]>(() => {
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

const onlineMembers = computed(() =>
	members.value.filter((m) => m.presence === 'ingame' || m.presence === 'launcher'),
)
const offlineMembers = computed(() =>
	members.value.filter((m) => !m.presence || m.presence === 'offline'),
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

const filteredOnline = computed(() => {
	const q = plainText(search.value).trim().toLowerCase()
	const list = onlineMembers.value
	if (!q) return list
	return list.filter((m) => m.minecraft_nick.toLowerCase().includes(q))
})

const filteredOffline = computed(() => {
	const q = plainText(search.value).trim().toLowerCase()
	const list = offlineMembers.value
	if (!q) return list
	return list.filter((m) => m.minecraft_nick.toLowerCase().includes(q))
})

const { avatarFor } = useLumenCommunityAvatars(members)

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
	if (presence === 'ingame') return 'presence-dot--ingame'
	if (presence === 'launcher') return 'presence-dot--launcher'
	return 'presence-dot--offline'
}

function presenceLabel(member: LumenCommunityMember) {
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

function createContextMenuOptions(member: LumenCommunityMember): ButtonMenuOption[] {
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

function openMemberContextMenu(event: MouseEvent, member: LumenCommunityMember) {
	memberOptions.value?.open(event, createContextMenuOptions(member))
}

function canJoin(member: LumenCommunityMember) {
	return canPlayWithFriend(member)
}

function canViewPack(member: LumenCommunityMember) {
	return canViewFriendPack(member)
}

async function viewFriendPack(member: LumenCommunityMember) {
	await playWithModal.value?.viewPack(member)
}

async function openPlayWith(member: LumenCommunityMember) {
	joiningId.value = member.id
	await playWithModal.value?.open(member)
}

function onPlayWithClosed() {
	joiningId.value = null
}

async function joinFriend(member: LumenCommunityMember) {
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
		id: 'Lumen.community.heading',
		defaultMessage: 'Znajomi',
	},
	tabFriends: {
		id: 'Lumen.community.tab.friends',
		defaultMessage: 'Znajomi',
	},
	tabChat: {
		id: 'Lumen.community.tab.chat',
		defaultMessage: 'Czat',
	},
	search: {
		id: 'Lumen.community.search',
		defaultMessage: 'Szukaj znajomych...',
	},
	signIn: {
		id: 'Lumen.community.sign-in',
		defaultMessage: 'Zaloguj się do Lumen, aby zobaczyć wszystkich graczy korzystających z tego launchera.',
	},
	signInAction: {
		id: 'Lumen.community.sign-in-action',
		defaultMessage: 'Zaloguj się',
	},
	registerAction: {
		id: 'Lumen.community.register-action',
		defaultMessage: 'Połącz konto',
	},
	empty: {
		id: 'Lumen.community.empty',
		defaultMessage: 'Nie masz jeszcze żadnych znajomych :(',
	},
	noMatch: {
		id: 'Lumen.community.no-match',
		defaultMessage: `Brak znajomych pasujących do ''{query}''`,
	},
	offline: {
		id: 'Lumen.community.offline',
		defaultMessage: 'Offline',
	},
	count: {
		id: 'Lumen.community.count',
		defaultMessage: '{count, plural, one {# gracz} other {# graczy}}',
	},
	connected: {
		id: 'Lumen.community.connected',
		defaultMessage: 'Połączono',
	},
	disconnected: {
		id: 'Lumen.community.disconnected',
		defaultMessage: 'Rozłączono',
	},
	inLauncher: {
		id: 'Lumen.community.in-launcher',
		defaultMessage: 'W launcherze',
	},
	inGame: {
		id: 'Lumen.community.in-game',
		defaultMessage: 'Gra: {name}',
	},
	inGameUnknown: {
		id: 'Lumen.community.in-game-unknown',
		defaultMessage: 'W grze',
	},
	lastSeen: {
		id: 'Lumen.community.last-seen',
		defaultMessage: 'Ostatnio widziany {time}',
	},
	join: {
		id: 'Lumen.community.join',
		defaultMessage: 'Dołącz',
	},
	playWith: {
		id: 'Lumen.community.play-with',
		defaultMessage: 'Graj z',
	},
	friendInGame: {
		id: 'Lumen.community.friend-in-game',
		defaultMessage: '{nick} jest w grze',
	},
	messagePlayer: {
		id: 'Lumen.community.message-player',
		defaultMessage: 'Wyślij wiadomość',
	},
	viewPack: {
		id: 'Lumen.community.view-pack',
		defaultMessage: 'Zobacz paczkę',
	},
	memberActionsLabel: {
		id: 'Lumen.community.actions.label',
		defaultMessage: 'Akcje znajomego',
	},
	onlineSection: {
		id: 'Lumen.community.section.online',
		defaultMessage: 'Online',
	},
	offlineSection: {
		id: 'Lumen.community.section.offline',
		defaultMessage: 'Offline',
	},
	showMore: {
		id: 'Lumen.community.show-more',
		defaultMessage: 'Pokaż więcej',
	},
	showLess: {
		id: 'Lumen.community.show-less',
		defaultMessage: 'Pokaż mniej',
	},
})
</script>

<template>
	<div class="community-panel flex h-full min-h-0 flex-1 flex-col">
		<ContextMenu ref="memberOptions" :label="formatMessage(messages.memberActionsLabel)" />
		<PlayWithFriendModal ref="playWithModal" @closed="onPlayWithClosed" />

		<div class="community-header" data-tauri-drag-region>
			<h3 class="community-heading" data-tauri-drag-region>
				{{ formatMessage(messages.heading) }}
			</h3>
			<div class="community-status">
				<span class="community-status-dot" :class="connected ? 'community-status-dot--online' : 'community-status-dot--offline'" />
				<span>{{ connected ? formatMessage(messages.connected) : formatMessage(messages.disconnected) }}</span>
				<span v-if="session && members.length > 0" class="community-count">
					{{ formatMessage(messages.count, { count: members.length }) }}
				</span>
			</div>
		</div>

		<template v-if="listLoading">
			<div class="community-skeleton">
				<div v-for="n in 5" :key="n" class="skeleton-row">
					<div class="skeleton-avatar" />
					<div class="skeleton-text">
						<div class="skeleton-line skeleton-line--short" />
						<div class="skeleton-line skeleton-line--long" />
					</div>
				</div>
			</div>
		</template>

		<template v-else-if="!session">
			<div class="community-signin">
				<p class="signin-text">
					{{ formatMessage(messages.signIn) }}
				</p>
				<div class="signin-actions">
					<Button type="colored" color="brand" class="w-full" @click="emit('signIn')">
						{{ formatMessage(messages.signInAction) }}
					</Button>
					<Button class="w-full" @click="emit('register')">
						{{ formatMessage(messages.registerAction) }}
					</Button>
				</div>
			</div>
		</template>

		<template v-else>
			<div class="community-tabs" role="tablist" :aria-label="formatMessage(messages.heading)">
				<button
					type="button"
					role="tab"
					class="community-tab"
					:class="{ 'community-tab--active': panelTab === 'friends' }"
					:aria-selected="panelTab === 'friends'"
					@click="panelTab = 'friends'"
				>
					{{ formatMessage(messages.tabFriends) }}
					<span v-if="onlineMembers.length > 0" class="tab-badge tab-badge--online">
						{{ onlineMembers.length }}
					</span>
				</button>
				<button
					type="button"
					role="tab"
					class="community-tab"
					:class="{ 'community-tab--active': panelTab === 'chat' }"
					:aria-selected="panelTab === 'chat'"
					@click="panelTab = 'chat'"
				>
					{{ formatMessage(messages.tabChat) }}
					<span
						v-if="badgeUnread > 0 && panelTab !== 'chat'"
						class="tab-badge tab-badge--unread"
					>
						{{ badgeUnread > 99 ? '99+' : badgeUnread }}
					</span>
				</button>
			</div>

			<template v-if="panelTab === 'friends'">
				<div v-if="members.length === 0" class="community-empty">
					<p>{{ formatMessage(messages.empty) }}</p>
				</div>

				<template v-else>
					<div class="community-search-wrapper">
						<SearchIcon class="community-search-icon" />
						<input
							v-model="search"
							type="text"
							class="community-search-input"
							:placeholder="formatMessage(messages.search)"
							@keyup.esc="search = ''"
						/>
					</div>

					<div v-if="filteredOnline.length > 0" class="community-section">
						<div class="community-section-header">
							<span class="community-section-title">
								{{ formatMessage(messages.onlineSection) }}
							</span>
							<span class="community-section-count">{{ filteredOnline.length }}</span>
						</div>
						<div class="community-list">
							<div
								v-for="(member, index) in filteredOnline"
								:key="member.id"
								class="community-card"
								:style="{ '--stagger': `${Math.min(index, 8) * 40}ms` }"
								@contextmenu.prevent.stop="(event) => openMemberContextMenu(event, member)"
							>
								<div class="community-card-avatar">
									<Avatar :src="avatarFor(member)" :alt="member.minecraft_nick" size="36px" circle />
									<span class="presence-dot" :class="presenceDotClass(member.presence)" />
								</div>
								<div class="community-card-info">
									<span class="community-card-name">{{ member.minecraft_nick }}</span>
									<span class="community-card-status" :class="{ 'community-card-status--ingame': member.presence === 'ingame' }">
										{{ presenceLabel(member) }}
									</span>
								</div>
								<IconButton
									v-if="canJoin(member)"
									v-tooltip="formatMessage(messages.playWith)"
									type="standard"
									color="brand"
									class="community-card-action"
									:label="formatMessage(messages.playWith)"
									:disabled="joiningId === member.id"
									@click="joinFriend(member)"
								>
									<PlayIcon />
								</IconButton>
							</div>
						</div>
					</div>

					<div
						v-if="filteredOffline.length > 0"
						class="community-section"
					>
						<button
							class="community-section-header community-section-header--clickable"
							@click="offlineExpanded = !offlineExpanded"
						>
							<span class="community-section-title">
								{{ formatMessage(messages.offlineSection) }}
							</span>
							<span class="community-section-count">{{ filteredOffline.length }}</span>
							<span class="community-section-toggle" :class="{ 'community-section-toggle--open': offlineExpanded }">
								&#9660;
							</span>
						</button>
						<Transition name="section-collapse">
							<div v-if="offlineExpanded" class="community-list community-list--offline">
								<div
									v-for="(member, index) in filteredOffline"
									:key="member.id"
									class="community-card community-card--offline"
									:style="{ '--stagger': `${Math.min(index, 8) * 30}ms` }"
									@contextmenu.prevent.stop="(event) => openMemberContextMenu(event, member)"
								>
									<div class="community-card-avatar">
										<Avatar :src="avatarFor(member)" :alt="member.minecraft_nick" size="32px" circle />
										<span class="presence-dot presence-dot--offline" />
									</div>
									<div class="community-card-info">
										<span class="community-card-name">{{ member.minecraft_nick }}</span>
										<span class="community-card-status">
											{{ presenceLabel(member) }}
										</span>
									</div>
								</div>
							</div>
						</Transition>
					</div>

					<p v-if="filteredOnline.length === 0 && filteredOffline.length === 0 && search" class="community-no-match">
						{{ formatMessage(messages.noMatch, { query: plainText(search) }) }}
					</p>
				</template>
			</template>

			<div v-else class="community-chat">
				<LumenChatPanel
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
.community-header {
	padding: 0.75rem 0.75rem 0.5rem;
}

.community-heading {
	margin: 0;
	font-size: 0.9rem;
	font-weight: 600;
	color: var(--color-contrast);
}

.community-status {
	display: flex;
	align-items: center;
	gap: 0.375rem;
	margin-top: 0.25rem;
	font-size: 0.7rem;
	color: var(--color-secondary);
}

.community-status-dot {
	width: 6px;
	height: 6px;
	border-radius: 50%;
	flex-shrink: 0;
}

.community-status-dot--online {
	background: #22c55e;
	box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
}

.community-status-dot--offline {
	background: var(--color-secondary);
	opacity: 0.4;
}

.community-count {
	margin-left: auto;
	opacity: 0.6;
}

.community-tabs {
	display: grid;
	grid-template-columns: 1fr 1fr;
	gap: 0;
	margin: 0 0.5rem;
	border-bottom: 1px solid var(--shell-border);
}

.community-tab {
	background: transparent;
	border: 0;
	border-bottom: 2px solid transparent;
	color: var(--color-secondary);
	cursor: pointer;
	display: flex;
	align-items: center;
	justify-content: center;
	gap: 0.375rem;
	font-size: 0.75rem;
	font-weight: 600;
	padding: 0.625rem 0.25rem;
	text-align: center;
	transition: color 0.2s, border-color 0.2s;
}

.community-tab:hover {
	color: var(--color-primary);
}

.community-tab--active {
	border-bottom-color: var(--color-brand);
	color: var(--color-brand);
	text-shadow: 0 0 12px color-mix(in srgb, var(--color-brand) 35%, transparent);
}

.tab-badge {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	min-width: 1rem;
	height: 1rem;
	padding: 0 0.25rem;
	border-radius: 9999px;
	font-size: 0.6rem;
	font-weight: 700;
	line-height: 1;
}

.tab-badge--online {
	background: rgba(34, 197, 94, 0.2);
	color: #22c55e;
}

.tab-badge--unread {
	background: var(--emerus-primary);
	color: #f1f4f2;
}

.community-search-wrapper {
	position: relative;
	margin: 0.5rem 0.5rem 0;
}

.community-search-icon {
	position: absolute;
	left: 0.625rem;
	top: 50%;
	transform: translateY(-50%);
	width: 14px;
	height: 14px;
	color: var(--color-secondary);
	pointer-events: none;
}

.community-search-input {
	width: 100%;
	box-sizing: border-box;
	padding: 0.5rem 0.625rem 0.5rem 2rem;
	background: var(--shell-card);
	border: 1px solid var(--shell-border);
	border-radius: 10px;
	color: var(--color-contrast);
	font-size: 0.75rem;
	outline: none;
	transition: border-color 0.2s, background 0.2s;
}

.community-search-input::placeholder {
	color: var(--color-secondary);
	opacity: 0.6;
}

.community-search-input:focus {
	border-color: color-mix(in srgb, var(--color-brand) 35%, transparent);
}

.community-section {
	margin-top: 0.5rem;
}

.community-section-header {
	display: flex;
	align-items: center;
	gap: 0.375rem;
	padding: 0.375rem 0.75rem;
	background: none;
	border: none;
	width: 100%;
	text-align: left;
}

.community-section-header--clickable {
	cursor: pointer;
	border-radius: 6px;
	transition: background 0.15s;
}

.community-section-header--clickable:hover {
	background: color-mix(in srgb, var(--color-brand) 8%, transparent);
}

.community-section-title {
	font-size: 0.65rem;
	font-weight: 700;
	text-transform: uppercase;
	letter-spacing: 0.05em;
	color: var(--color-secondary);
}

.community-section-count {
	font-size: 0.6rem;
	color: var(--color-secondary);
	opacity: 0.5;
}

.community-section-toggle {
	font-size: 0.5rem;
	color: var(--color-secondary);
	opacity: 0.4;
	transition: transform 0.2s;
	margin-left: auto;
}

.community-section-toggle--open {
	transform: rotate(180deg);
}

.community-list {
	display: flex;
	flex-direction: column;
	padding: 0 0.375rem;
}

.community-list--offline {
	padding-bottom: 0.25rem;
}

.community-card {
	display: grid;
	grid-template-columns: auto 1fr auto;
	align-items: center;
	gap: 0.5rem;
	padding: 0.375rem 0.5rem;
	margin: 0.0625rem 0;
	border-radius: 10px;
	cursor: default;
	transition: background 0.15s, transform 0.15s, box-shadow 0.2s;
	animation: card-in 0.3s cubic-bezier(0.32, 0.72, 0, 1) both;
	animation-delay: var(--stagger, 0ms);
}

.community-card:hover {
	background: color-mix(in srgb, var(--color-brand) 8%, transparent);
}

.community-card--offline {
	opacity: 0.55;
	transition: opacity 0.2s, background 0.15s;
}

.community-card--offline:hover {
	opacity: 0.75;
}

@keyframes card-in {
	from {
		opacity: 0;
		transform: translateY(4px);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}

.community-card-avatar {
	position: relative;
	flex-shrink: 0;
}

.presence-dot {
	position: absolute;
	bottom: -1px;
	right: -1px;
	width: 10px;
	height: 10px;
	border-radius: 50%;
	border: 2px solid var(--shell-panel-strong);
}

.presence-dot--ingame {
	background: var(--emerus-accent-bright);
}

.presence-dot--launcher {
	background: #22c55e;
	box-shadow: 0 0 4px rgba(34, 197, 94, 0.3);
}

.presence-dot--offline {
	background: #6b7280;
}

.community-card-info {
	display: flex;
	flex-direction: column;
	min-width: 0;
}

.community-card-name {
	font-size: 0.8rem;
	font-weight: 600;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.community-card-status {
	font-size: 0.675rem;
	color: var(--color-secondary);
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.community-card-status--ingame {
	color: var(--color-brand);
}

.community-card-action {
	flex-shrink: 0;
	opacity: 0;
	transform: scale(0.85);
	transition: opacity 0.15s, transform 0.15s;
}

.community-card:hover .community-card-action {
	opacity: 1;
	transform: scale(1);
}

.community-signin {
	padding: 1rem 0.75rem;
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.signin-text {
	margin: 0;
	font-size: 0.8rem;
	text-align: center;
	color: var(--color-secondary);
	line-height: 1.4;
}

.signin-actions {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.community-empty {
	padding: 2rem 0.75rem;
	text-align: center;
}

.community-empty p {
	margin: 0;
	font-size: 0.8rem;
	color: var(--color-secondary);
}

.community-no-match {
	padding: 1rem 0.75rem;
	margin: 0;
	font-size: 0.8rem;
	text-align: center;
	color: var(--color-secondary);
}

.community-chat {
	display: flex;
	flex-direction: column;
	flex: 1;
	min-height: 0;
}

.community-skeleton {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: 0.5rem 0.75rem;
}

.skeleton-row {
	display: flex;
	gap: 0.5rem;
	align-items: center;
	animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-avatar {
	width: 36px;
	height: 36px;
	border-radius: 50%;
	background: var(--shell-border);
}

.skeleton-text {
	flex: 1;
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
}

.skeleton-line {
	height: 8px;
	border-radius: 4px;
	background: var(--shell-border);
}

.skeleton-line--short {
	width: 50%;
}

.skeleton-line--long {
	width: 75%;
}

@keyframes skeleton-pulse {
	0%, 100% {
		opacity: 1;
	}
	50% {
		opacity: 0.4;
	}
}

.section-collapse-enter-active,
.section-collapse-leave-active {
	transition: all 0.2s cubic-bezier(0.32, 0.72, 0, 1);
	overflow: hidden;
}

.section-collapse-enter-from,
.section-collapse-leave-to {
	opacity: 0;
	max-height: 0;
}

.section-collapse-enter-to,
.section-collapse-leave-from {
	opacity: 1;
	max-height: 500px;
}
</style>
