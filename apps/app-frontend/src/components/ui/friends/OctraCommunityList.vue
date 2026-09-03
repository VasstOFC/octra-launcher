<script setup lang="ts">
import {
	MessageIcon,
	PackageSearchIcon,
	PlayIcon,
	PlusIcon,
	SearchIcon,
	ServerIcon,
	ShareIcon,
	TrashIcon,
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
import { computed, ref, useTemplateRef, watch } from 'vue'

import PlayWithFriendModal from '@/components/ui/friends/PlayWithFriendModal.vue'
import {
	canPlayWithFriend,
	canViewFriendPack,
} from '@/composables/play-with-friend'
import { handleSevereError } from '@/composables/use-error.js'
import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { list } from '@/helpers/instance'
import {
	octraCommunity,
	octraSharedServersAdd,
	octraSharedServersDelete,
	octraSharedServersList,
	octraShareJoinAddress,
} from '@/helpers/octra-account.js'
import { start_join_server } from '@/helpers/worlds'

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

type OctraSharedServer = {
	id: number
	name: string
	address: string
	created_by: number
	created_by_nick?: string | null
	created_at: string
}

const props = defineProps<{
	session: OctraAccountSession | null
	loadingSession?: boolean
}>()

const emit = defineEmits<{
	signIn: []
	register: []
	messagePlayer: [userId: number]
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime({ numeric: 'auto', style: 'short' })
const search = ref('')
const panelTab = ref<'friends' | 'servers'>('friends')
const joiningId = ref<number | null>(null)
const joiningSharedId = ref<number | null>(null)
const shareAddress = ref('')
const sharingAddress = ref(false)
const sharedServers = ref<OctraSharedServer[]>([])
const sharedName = ref('')
const sharedAddress = ref('')
const addingShared = ref(false)
const previousPresence = ref<Map<number, string>>(new Map())
const presenceReady = ref(false)
const memberOptions = useTemplateRef('memberOptions')
const playWithModal = useTemplateRef<InstanceType<typeof PlayWithFriendModal>>('playWithModal')

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

watch(
	() => props.session?.username ?? null,
	() => {
		previousPresence.value = new Map()
		presenceReady.value = false
		panelTab.value = 'friends'
		void refreshSharedServers()
	},
)

async function refreshSharedServers() {
	if (!props.session) {
		sharedServers.value = []
		return
	}
	try {
		sharedServers.value = await octraSharedServersList()
	} catch {
		sharedServers.value = []
	}
}

void refreshSharedServers()

function presenceDotClass(presence?: string) {
	if (presence === 'ingame') return 'bg-brand-green'
	if (presence === 'launcher') return 'bg-brand'
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
			action: () => emit('messagePlayer', member.id),
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

function pickInstance(
	instances: Awaited<ReturnType<typeof list>>,
	preferredName: string | null | undefined,
) {
	if (instances.length === 0) return null
	const preferred = preferredName?.trim().toLowerCase()
	if (preferred) {
		const byName = instances.find((instance) => instance.name.toLowerCase() === preferred)
		if (byName) return byName
	}
	return [...instances].sort((a, b) => {
		const aTime = a.last_played ? new Date(a.last_played).getTime() : 0
		const bTime = b.last_played ? new Date(b.last_played).getTime() : 0
		return bTime - aTime
	})[0] ?? null
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

async function shareMyIp() {
	const address = shareAddress.value.trim()
	if (!address || sharingAddress.value) return
	sharingAddress.value = true
	try {
		await octraShareJoinAddress(address)
		addNotification({
			title: formatMessage(messages.shareIpSuccess),
			text: '',
			type: 'success',
		})
		shareAddress.value = ''
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		sharingAddress.value = false
	}
}

async function addSharedServer() {
	const name = sharedName.value.trim()
	const address = sharedAddress.value.trim()
	if (!name || !address || addingShared.value) return
	addingShared.value = true
	try {
		await octraSharedServersAdd(name, address)
		sharedName.value = ''
		sharedAddress.value = ''
		await refreshSharedServers()
		addNotification({
			title: formatMessage(messages.sharedAddSuccess),
			text: '',
			type: 'success',
		})
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		addingShared.value = false
	}
}

async function deleteSharedServer(server: OctraSharedServer) {
	try {
		await octraSharedServersDelete(server.id)
		await refreshSharedServers()
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

async function joinSharedServer(server: OctraSharedServer) {
	joiningSharedId.value = server.id
	try {
		const instances = await list()
		const instance = pickInstance(instances, null)
		if (!instance) {
			handleError(formatMessage(messages.noInstance))
			return
		}
		await start_join_server(instance.id, server.address)
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		joiningSharedId.value = null
	}
}

const messages = defineMessages({
	heading: {
		id: 'octra.community.heading',
		defaultMessage: 'Friends',
	},
	tabFriends: {
		id: 'octra.community.tab.friends',
		defaultMessage: 'Friends',
	},
	tabServers: {
		id: 'octra.community.tab.servers',
		defaultMessage: 'Servers',
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
	shareIpLabel: {
		id: 'octra.community.share-ip',
		defaultMessage: 'Share my server IP…',
	},
	shareIpPlaceholder: {
		id: 'octra.community.share-ip-placeholder',
		defaultMessage: 'play.example.com:25565',
	},
	shareIpAction: {
		id: 'octra.community.share-ip-action',
		defaultMessage: 'Share IP',
	},
	shareIpSuccess: {
		id: 'octra.community.share-ip-success',
		defaultMessage: 'Your join address is shared with friends.',
	},
	noInstance: {
		id: 'octra.community.no-instance',
		defaultMessage: 'Create or install an instance before joining a friend.',
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
	sharedHeading: {
		id: 'octra.community.shared-heading',
		defaultMessage: 'Shared servers',
	},
	sharedEmpty: {
		id: 'octra.community.shared-empty',
		defaultMessage: 'No shared servers yet.',
	},
	sharedNamePlaceholder: {
		id: 'octra.community.shared-name',
		defaultMessage: 'Name',
	},
	sharedAddressPlaceholder: {
		id: 'octra.community.shared-address',
		defaultMessage: 'Address',
	},
	sharedAdd: {
		id: 'octra.community.shared-add',
		defaultMessage: 'Add server',
	},
	sharedAddSuccess: {
		id: 'octra.community.shared-add-success',
		defaultMessage: 'Shared server added.',
	},
	sharedJoin: {
		id: 'octra.community.shared-join',
		defaultMessage: 'Join server',
	},
	sharedDelete: {
		id: 'octra.community.shared-delete',
		defaultMessage: 'Delete server',
	},
})
</script>

<template>
	<div class="flex flex-col gap-3">
		<ContextMenu ref="memberOptions" :label="formatMessage(messages.memberActionsLabel)" />
		<PlayWithFriendModal ref="playWithModal" @closed="onPlayWithClosed" />
		<div class="flex items-start justify-between gap-2">
			<div class="min-w-0">
				<h3 class="m-0 text-base font-medium text-primary">
					{{ formatMessage(messages.heading) }}
				</h3>
				<div class="mt-0.5 flex items-center gap-1.5 text-[11px] leading-none text-secondary">
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
				class="grid grid-cols-2 gap-0.5 rounded-lg bg-surface-2 p-0.5"
				role="tablist"
				:aria-label="formatMessage(messages.heading)"
			>
				<button
					type="button"
					role="tab"
					class="rounded-md px-2 py-1.5 text-xs font-medium transition-colors"
					:class="
						panelTab === 'friends'
							? 'bg-surface-3 text-contrast'
							: 'text-secondary hover:text-primary'
					"
					:aria-selected="panelTab === 'friends'"
					@click="panelTab = 'friends'"
				>
					{{ formatMessage(messages.tabFriends) }}
				</button>
				<button
					type="button"
					role="tab"
					class="rounded-md px-2 py-1.5 text-xs font-medium transition-colors"
					:class="
						panelTab === 'servers'
							? 'bg-surface-3 text-contrast'
							: 'text-secondary hover:text-primary'
					"
					:aria-selected="panelTab === 'servers'"
					@click="panelTab = 'servers'"
				>
					{{ formatMessage(messages.tabServers) }}
					<span
						v-if="sharedServers.length > 0"
						class="ml-1 text-[10px] text-secondary"
					>
						{{ sharedServers.length }}
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
					<div class="community-list flex flex-col gap-0.5">
						<div
							v-for="(member, index) in filtered"
							:key="member.id"
							class="community-row grid grid-cols-[auto_1fr_auto] items-center gap-2 rounded-lg px-1 py-1.5 select-none hover:bg-surface-3"
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

			<template v-else>
				<div class="flex flex-col gap-1.5 rounded-lg bg-surface-2 p-2">
					<span class="text-[11px] font-medium text-secondary">
						{{ formatMessage(messages.shareIpLabel) }}
					</span>
					<div class="flex items-center gap-1.5">
						<input
							v-model="shareAddress"
							type="text"
							class="min-h-8 w-full rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-1 text-xs text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.shareIpPlaceholder)"
							:disabled="sharingAddress"
							@keydown.enter.prevent="shareMyIp"
						/>
						<IconButton
							v-tooltip="formatMessage(messages.shareIpAction)"
							type="standard"
							color="brand"
							:label="formatMessage(messages.shareIpAction)"
							:disabled="sharingAddress || !shareAddress.trim()"
							@click="shareMyIp"
						>
							<ShareIcon />
						</IconButton>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<div class="flex items-center gap-1.5 text-primary">
						<ServerIcon class="size-3.5 shrink-0 text-secondary" />
						<h4 class="m-0 text-sm font-medium">
							{{ formatMessage(messages.sharedHeading) }}
						</h4>
					</div>
					<p v-if="sharedServers.length === 0" class="m-0 text-xs text-secondary">
						{{ formatMessage(messages.sharedEmpty) }}
					</p>
					<div
						v-for="server in sharedServers"
						:key="server.id"
						class="grid grid-cols-[1fr_auto] items-center gap-1 rounded-lg px-1 py-1 hover:bg-surface-3"
					>
						<div class="min-w-0">
							<span class="block truncate text-sm text-contrast">{{ server.name }}</span>
							<span class="block truncate text-[11px] text-secondary">{{ server.address }}</span>
						</div>
						<div class="flex items-center gap-0.5">
							<IconButton
								v-tooltip="formatMessage(messages.sharedJoin)"
								type="quiet"
								color="brand"
								:label="formatMessage(messages.sharedJoin)"
								:disabled="joiningSharedId === server.id"
								@click="joinSharedServer(server)"
							>
								<PlayIcon />
							</IconButton>
							<IconButton
								v-tooltip="formatMessage(messages.sharedDelete)"
								type="quiet"
								:label="formatMessage(messages.sharedDelete)"
								@click="deleteSharedServer(server)"
							>
								<TrashIcon />
							</IconButton>
						</div>
					</div>
					<div class="flex flex-col gap-1.5">
						<input
							v-model="sharedName"
							type="text"
							class="min-h-8 w-full rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-1 text-xs text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.sharedNamePlaceholder)"
						/>
						<input
							v-model="sharedAddress"
							type="text"
							class="min-h-8 w-full rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-1 text-xs text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.sharedAddressPlaceholder)"
							@keydown.enter.prevent="addSharedServer"
						/>
						<Button
							type="colored"
							color="brand"
							class="w-full"
							:disabled="addingShared || !sharedName.trim() || !sharedAddress.trim()"
							@click="addSharedServer"
						>
							<PlusIcon />
							{{ formatMessage(messages.sharedAdd) }}
						</Button>
					</div>
				</div>
			</template>
		</template>
	</div>
</template>

<style scoped>
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
