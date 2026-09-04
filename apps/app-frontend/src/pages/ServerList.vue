<script setup lang="ts">
import {
	ClipboardCopyIcon,
	GlobeIcon,
	PackageSearchIcon,
	PlayIcon,
	PlusIcon,
	RefreshCwIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ConfirmModal,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onActivated, ref, useTemplateRef, watch } from 'vue'

import PlayWithFriendModal from '@/components/ui/friends/PlayWithFriendModal.vue'
import { useAppEvent } from '@/composables/use-app-event'
import {
	canPlayWithFriend,
	canViewFriendPack,
	type PlayWithFriendMember,
} from '@/composables/play-with-friend'
import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { toError } from '@/helpers/errors'
import { list as listInstances } from '@/helpers/instance'
import {
	octraAccountSession,
	octraCommunity,
	octraSharedServersAdd,
	octraSharedServersDelete,
	octraSharedServersList,
} from '@/helpers/octra-account.js'
import {
	get_instance_worlds,
	get_server_status,
	remove_server_from_instance,
	type ServerWorld,
	start_join_server,
} from '@/helpers/worlds'
import { instanceKeys } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

defineOptions({ name: 'ServerListPage' })

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

type SharedServerRow = {
	key: string
	name: string
	address: string
	sharedId: number
	createdBy: number
	createdByNick: string | null
}

type LocalServerRow = {
	key: string
	name: string
	address: string
	index: number
}

type PingState = {
	online: boolean
	ping?: number
	playersOnline?: number
	playersMax?: number
	version?: string
}

const messages = defineMessages({
	title: {
		id: 'app.servers.title',
		defaultMessage: 'Servers',
	},
	subtitle: {
		id: 'app.servers.subtitle',
		defaultMessage: 'Join friends in game, or hop onto a saved address.',
	},
	quickJoin: {
		id: 'app.servers.quick-join',
		defaultMessage: 'Quick join',
	},
	quickJoinPlaceholder: {
		id: 'app.servers.quick-join.placeholder',
		defaultMessage: 'play.example.com:25565',
	},
	quickJoinInstance: {
		id: 'app.servers.quick-join.instance',
		defaultMessage: 'Instance',
	},
	quickJoinAction: {
		id: 'app.servers.quick-join.action',
		defaultMessage: 'Join',
	},
	friendsInGame: {
		id: 'app.servers.friends-ingame.title',
		defaultMessage: 'Friends in game',
	},
	friendsInGameEmpty: {
		id: 'app.servers.friends-ingame.empty',
		defaultMessage: 'Nobody from Octra is in game with a join address right now.',
	},
	friendsSignIn: {
		id: 'app.servers.friends-ingame.sign-in',
		defaultMessage: 'Sign in to Octra in the friends panel to see who is playing.',
	},
	playingAs: {
		id: 'app.servers.friends-ingame.playing',
		defaultMessage: 'Playing: {name}',
	},
	playingUnknown: {
		id: 'app.servers.friends-ingame.playing-unknown',
		defaultMessage: 'In game',
	},
	playWith: {
		id: 'app.servers.friends-ingame.play-with',
		defaultMessage: 'Join',
	},
	viewPack: {
		id: 'app.servers.friends-ingame.view-pack',
		defaultMessage: 'View pack',
	},
	sharedTitle: {
		id: 'app.servers.shared.title',
		defaultMessage: 'Shared servers',
	},
	sharedSubtitle: {
		id: 'app.servers.shared.subtitle',
		defaultMessage:
			'Shown at the top of the Minecraft multiplayer list with the Octra icon.',
	},
	sharedEmpty: {
		id: 'app.servers.shared.empty',
		defaultMessage: 'No shared servers yet. Share an address below so friends can join.',
	},
	sharedEmptySignedOut: {
		id: 'app.servers.shared.empty-signed-out',
		defaultMessage: 'Sign in to Octra to share servers with friends.',
	},
	sharedBy: {
		id: 'app.servers.shared.by',
		defaultMessage: 'Shared by {name}',
	},
	sharedNamePlaceholder: {
		id: 'app.servers.shared.name-placeholder',
		defaultMessage: 'Server name',
	},
	sharedAddressPlaceholder: {
		id: 'app.servers.shared.address-placeholder',
		defaultMessage: 'play.example.com:25565',
	},
	sharedAdd: {
		id: 'app.servers.shared.add',
		defaultMessage: 'Share server',
	},
	sharedAdded: {
		id: 'app.servers.shared.added',
		defaultMessage: 'Server shared with everyone',
	},
	shareLocal: {
		id: 'app.servers.shared.share-local',
		defaultMessage: 'Share with friends',
	},
	sharedDeleteConfirmTitle: {
		id: 'app.servers.shared.delete-confirm.title',
		defaultMessage: 'Remove shared server?',
	},
	sharedDeleteConfirmBody: {
		id: 'app.servers.shared.delete-confirm.body',
		defaultMessage:
			'This shared server will disappear for everyone who has it. This cannot be undone.',
	},
	sharedDeleteConfirmAction: {
		id: 'app.servers.shared.delete-confirm.action',
		defaultMessage: 'Remove for everyone',
	},
	localTitle: {
		id: 'app.servers.local.title',
		defaultMessage: 'My local servers',
	},
	localSubtitle: {
		id: 'app.servers.local.subtitle',
		defaultMessage:
			'From Minecraft for the selected instance. Updates live while the game is open; deletions sync when Minecraft saves the list.',
	},
	localEmpty: {
		id: 'app.servers.local.empty',
		defaultMessage: 'No servers in this instance yet. Add one in Minecraft Multiplayer.',
	},
	localEmptyNoInstance: {
		id: 'app.servers.local.empty-no-instance',
		defaultMessage: 'Select an instance to see its Minecraft server list.',
	},
	savedDelete: {
		id: 'app.servers.saved.delete',
		defaultMessage: 'Remove server',
	},
	savedDeleted: {
		id: 'app.servers.saved.deleted',
		defaultMessage: 'Server removed',
	},
	refresh: {
		id: 'app.servers.refresh',
		defaultMessage: 'Refresh',
	},
	online: {
		id: 'app.servers.online',
		defaultMessage: 'Online',
	},
	offline: {
		id: 'app.servers.offline',
		defaultMessage: 'Offline',
	},
	copied: {
		id: 'app.servers.copied',
		defaultMessage: 'Address copied',
	},
	copy: {
		id: 'app.servers.copy',
		defaultMessage: 'Copy address',
	},
	play: {
		id: 'app.servers.play',
		defaultMessage: 'Play',
	},
	players: {
		id: 'app.servers.players',
		defaultMessage: '{online}/{max} players',
	},
	ping: {
		id: 'app.servers.ping',
		defaultMessage: '{ms} ms',
	},
	noInstance: {
		id: 'app.servers.no-instance',
		defaultMessage: 'Create or import an instance first to join a server.',
	},
	loading: {
		id: 'app.servers.loading',
		defaultMessage: 'Loading…',
	},
})

const breadcrumb = useRootBreadcrumb({
	slot: 'root',
	id: 'servers',
	label: formatMessage(messages.title),
	to: '/servers',
	visual: { type: 'icon', component: GlobeIcon },
})
onActivated(breadcrumb.reset)

const playWithModal = useTemplateRef<InstanceType<typeof PlayWithFriendModal>>('playWithModal')
const deleteSharedModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('deleteSharedModal')
const quickAddress = ref('')
const selectedInstanceId = ref<string | null>(null)
const joiningQuick = ref(false)
const joiningFriendId = ref<number | null>(null)
const joiningSavedKey = ref<string | null>(null)
const pinging = ref(false)
const pings = ref<Record<string, PingState>>({})
const shareName = ref('')
const shareAddress = ref('')
const sharing = ref(false)
const pendingDeleteShared = ref<SharedServerRow | null>(null)

const sessionQuery = useQuery({
	queryKey: ['octra', 'account-session'],
	queryFn: () => octraAccountSession(),
	staleTime: 30_000,
})

const communityQuery = useQuery({
	queryKey: computed(() => ['octra-community', sessionQuery.data.value?.username ?? null]),
	queryFn: () => octraCommunity(),
	enabled: computed(() => !!sessionQuery.data.value),
	staleTime: 10_000,
	refetchInterval: 15_000,
})

const sharedServersQuery = useQuery({
	queryKey: computed(() => ['octra', 'shared-servers', sessionQuery.data.value?.username ?? null]),
	queryFn: () => octraSharedServersList(),
	enabled: computed(() => !!sessionQuery.data.value),
	refetchInterval: 20_000,
})

const instancesQuery = useQuery({
	queryKey: instanceKeys.list(),
	queryFn: listInstances,
})

const playableInstances = computed(() => {
	const instances = instancesQuery.data.value ?? []
	return [...instances]
		.filter(
			(instance) =>
				instance.install_stage === 'installed' || instance.install_stage === 'pack_installed',
		)
		.sort((a, b) => {
			const aTime = a.last_played ? new Date(a.last_played).getTime() : 0
			const bTime = b.last_played ? new Date(b.last_played).getTime() : 0
			return bTime - aTime
		})
})

const defaultInstanceId = computed(() => playableInstances.value[0]?.id ?? null)

const resolvedInstanceId = computed(
	() =>
		selectedInstanceId.value &&
		playableInstances.value.some((instance) => instance.id === selectedInstanceId.value)
			? selectedInstanceId.value
			: defaultInstanceId.value,
)

watch(
	defaultInstanceId,
	(id) => {
		if (!selectedInstanceId.value && id) {
			selectedInstanceId.value = id
		}
	},
	{ immediate: true },
)

const localWorldsQuery = useQuery({
	queryKey: computed(() => instanceKeys.worlds(resolvedInstanceId.value ?? '')),
	queryFn: () => get_instance_worlds(resolvedInstanceId.value!),
	enabled: computed(() => !!resolvedInstanceId.value),
	staleTime: 0,
	refetchInterval: 4_000,
})

useAppEvent('instance', async (event) => {
	if (event.event !== 'servers_updated') return
	if (!resolvedInstanceId.value || event.instance_id !== resolvedInstanceId.value) return
	await queryClient.invalidateQueries({ queryKey: instanceKeys.worlds(resolvedInstanceId.value) })
})

const friendsInGame = computed(() => {
	const members = communityQuery.data.value?.members ?? []
	return members
		.filter((member) => canPlayWithFriend(member))
		.slice()
		.sort((a, b) =>
			a.minecraft_nick.localeCompare(b.minecraft_nick, undefined, { sensitivity: 'base' }),
		)
})

const { avatarFor } = useOctraCommunityAvatars(friendsInGame)

const selfNick = computed(() =>
	(sessionQuery.data.value?.minecraft_nick ?? '').trim().toLowerCase(),
)

const sharedServers = computed<SharedServerRow[]>(() => {
	const rows: SharedServerRow[] = []
	for (const server of sharedServersQuery.data.value ?? []) {
		const address = String(server.address ?? '').trim()
		if (!address) continue
		rows.push({
			key: `shared:${server.id}`,
			name: server.name || address,
			address,
			sharedId: Number(server.id),
			createdBy: Number(server.created_by),
			createdByNick: server.created_by_nick ? String(server.created_by_nick) : null,
		})
	}
	return rows.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }))
})

const localServers = computed<LocalServerRow[]>(() => {
	const worlds = localWorldsQuery.data.value ?? []
	const rows: LocalServerRow[] = []
	for (const world of worlds) {
		if (world.type !== 'server') continue
		const server = world as ServerWorld
		const address = server.address.trim()
		if (!address) continue
		// Octra shared servers are injected into servers.dat for in-game display;
		// keep them out of the local list (they already appear under Shared).
		if (server.name.trim().startsWith('[Octra] ')) continue
		rows.push({
			key: `local:${server.index}:${address.toLowerCase()}`,
			name: server.name || address,
			address,
			index: server.index,
		})
	}
	return rows
})

const pageLoading = computed(
	() =>
		instancesQuery.isPending.value ||
		(!!sessionQuery.data.value && communityQuery.isPending.value),
)

function isSharedByOther(server: SharedServerRow): boolean {
	const nick = (server.createdByNick ?? '').trim().toLowerCase()
	if (!nick || !selfNick.value) return !!nick
	return nick !== selfNick.value
}

async function refreshPings() {
	const addresses = [
		...new Set([
			...sharedServers.value.map((server) => server.address),
			...localServers.value.map((server) => server.address),
			...friendsInGame.value
				.map((member) => member.join_address?.trim())
				.filter((address): address is string => !!address),
		]),
	]
	if (addresses.length === 0) return
	pinging.value = true
	try {
		await Promise.all(
			addresses.map(async (address) => {
				try {
					const status = await get_server_status(address, null)
					pings.value = {
						...pings.value,
						[address]: {
							online: true,
							ping: status.ping,
							playersOnline: status.players?.online,
							playersMax: status.players?.max,
							version: status.version?.name,
						},
					}
				} catch {
					pings.value = {
						...pings.value,
						[address]: { online: false },
					}
				}
			}),
		)
	} finally {
		pinging.value = false
	}
}

async function copyAddress(address: string) {
	try {
		await navigator.clipboard.writeText(address)
		addNotification({
			title: formatMessage(messages.copied),
			text: address,
			type: 'success',
		})
	} catch (error: unknown) {
		handleError(toError(error))
	}
}

async function playAddress(address: string, busyKey?: string) {
	const instanceId = resolvedInstanceId.value
	if (!instanceId) {
		addNotification({
			title: formatMessage(messages.noInstance),
			text: '',
			type: 'error',
		})
		return
	}
	if (busyKey) joiningSavedKey.value = busyKey
	try {
		await start_join_server(instanceId, address)
	} catch (error: unknown) {
		handleError(toError(error))
	} finally {
		joiningSavedKey.value = null
	}
}

async function shareServer(name: string, address: string) {
	const trimmedName = name.trim() || address.trim()
	const trimmedAddress = address.trim()
	if (!trimmedAddress || sharing.value) return
	if (!sessionQuery.data.value) {
		addNotification({
			title: formatMessage(messages.sharedEmptySignedOut),
			text: '',
			type: 'error',
		})
		return
	}
	sharing.value = true
	try {
		await octraSharedServersAdd(trimmedName, trimmedAddress)
		await queryClient.invalidateQueries({ queryKey: ['octra', 'shared-servers'] })
		addNotification({
			title: formatMessage(messages.sharedAdded),
			text: trimmedAddress,
			type: 'success',
		})
		shareName.value = ''
		shareAddress.value = ''
	} catch (error: unknown) {
		handleError(toError(error))
	} finally {
		sharing.value = false
	}
}

async function shareFromForm() {
	await shareServer(shareName.value, shareAddress.value)
}

async function shareLocalServer(server: LocalServerRow) {
	await shareServer(server.name, server.address)
}

function requestDeleteShared(server: SharedServerRow) {
	pendingDeleteShared.value = server
	deleteSharedModal.value?.show()
}

async function confirmDeleteShared() {
	const server = pendingDeleteShared.value
	pendingDeleteShared.value = null
	if (!server) return
	joiningSavedKey.value = server.key
	try {
		await octraSharedServersDelete(server.sharedId)
		await queryClient.invalidateQueries({ queryKey: ['octra', 'shared-servers'] })
		addNotification({
			title: formatMessage(messages.savedDeleted),
			text: server.name,
			type: 'success',
		})
	} catch (error: unknown) {
		handleError(toError(error))
	} finally {
		joiningSavedKey.value = null
	}
}

async function deleteLocalServer(server: LocalServerRow) {
	const instanceId = resolvedInstanceId.value
	if (!instanceId) return
	joiningSavedKey.value = server.key
	try {
		await remove_server_from_instance(instanceId, server.index)
		await queryClient.invalidateQueries({ queryKey: instanceKeys.worlds(instanceId) })
		addNotification({
			title: formatMessage(messages.savedDeleted),
			text: server.name,
			type: 'success',
		})
	} catch (error: unknown) {
		handleError(toError(error))
	} finally {
		joiningSavedKey.value = null
	}
}

async function quickJoin() {
	const address = quickAddress.value.trim()
	if (!address || joiningQuick.value) return
	joiningQuick.value = true
	try {
		await playAddress(address)
	} finally {
		joiningQuick.value = false
	}
}

async function playWithFriend(member: PlayWithFriendMember) {
	joiningFriendId.value = member.id
	await playWithModal.value?.open(member)
}

function onPlayWithClosed() {
	joiningFriendId.value = null
}

async function viewPack(member: PlayWithFriendMember) {
	await playWithModal.value?.viewPack(member)
}
</script>

<template>
	<div class="flex h-full min-h-0 flex-col">
		<PlayWithFriendModal ref="playWithModal" @closed="onPlayWithClosed" />
		<ConfirmModal
			ref="deleteSharedModal"
			:title="formatMessage(messages.sharedDeleteConfirmTitle)"
			:description="formatMessage(messages.sharedDeleteConfirmBody)"
			:proceed-label="formatMessage(messages.sharedDeleteConfirmAction)"
			:markdown="false"
			@proceed="confirmDeleteShared"
		/>

		<div class="border-0 border-b border-solid border-surface-5 px-6 py-5">
			<div class="flex flex-wrap items-start justify-between gap-3">
				<div class="min-w-0">
					<h1 class="m-0 text-2xl font-semibold text-contrast">
						{{ formatMessage(messages.title) }}
					</h1>
					<p class="m-0 mt-1 text-sm text-secondary">{{ formatMessage(messages.subtitle) }}</p>
				</div>
				<Button
					:disabled="
						pinging ||
						(sharedServers.length === 0 &&
							localServers.length === 0 &&
							friendsInGame.length === 0)
					"
					@click="refreshPings"
				>
					<SpinnerIcon v-if="pinging" class="animate-spin" />
					<RefreshCwIcon v-else />
					{{ formatMessage(messages.refresh) }}
				</Button>
			</div>

			<div class="mt-4 flex flex-col gap-2 rounded-lg bg-surface-2 p-3">
				<span class="text-xs font-medium uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.quickJoin) }}
				</span>
				<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
					<input
						v-model="quickAddress"
						type="text"
						class="min-h-9 min-w-0 flex-1 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm text-primary placeholder:text-secondary"
						:placeholder="formatMessage(messages.quickJoinPlaceholder)"
						:disabled="joiningQuick"
						@keydown.enter.prevent="quickJoin"
					/>
					<select
						v-model="selectedInstanceId"
						class="min-h-9 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm text-primary sm:max-w-[14rem]"
						:disabled="playableInstances.length === 0"
						:aria-label="formatMessage(messages.quickJoinInstance)"
					>
						<option :value="null" disabled>
							{{ formatMessage(messages.quickJoinInstance) }}
						</option>
						<option
							v-for="instance in playableInstances"
							:key="instance.id"
							:value="instance.id"
						>
							{{ instance.name }}
						</option>
					</select>
					<Button
						type="colored"
						color="brand"
						:disabled="joiningQuick || !quickAddress.trim() || !resolvedInstanceId"
						@click="quickJoin"
					>
						<PlayIcon />
						{{ formatMessage(messages.quickJoinAction) }}
					</Button>
				</div>
			</div>
		</div>

		<div class="min-h-0 flex-1 overflow-auto p-6">
			<div
				v-if="pageLoading"
				class="flex items-center justify-center gap-2 py-16 text-sm text-secondary"
			>
				<SpinnerIcon class="animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>

			<template v-else>
				<section class="mx-auto max-w-3xl">
					<h2 class="m-0 text-xs font-medium uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.friendsInGame) }}
					</h2>

					<p
						v-if="!sessionQuery.data.value"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.friendsSignIn) }}
					</p>
					<p
						v-else-if="friendsInGame.length === 0"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.friendsInGameEmpty) }}
					</p>
					<ul v-else class="m-0 mt-2 flex list-none flex-col gap-1 p-0">
						<li
							v-for="friend in friendsInGame"
							:key="friend.id"
							class="flex flex-wrap items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-3"
						>
							<Avatar
								:src="avatarFor(friend)"
								:alt="friend.minecraft_nick"
								size="36px"
								circle
							/>
							<div class="min-w-0 flex-1">
								<p class="m-0 truncate text-sm font-medium text-contrast">
									{{ friend.minecraft_nick }}
								</p>
								<p class="m-0 truncate text-xs text-secondary">
									{{
										friend.instance_name
											? formatMessage(messages.playingAs, { name: friend.instance_name })
											: formatMessage(messages.playingUnknown)
									}}
									<template v-if="friend.join_address && pings[friend.join_address]?.online">
										·
										{{
											formatMessage(messages.ping, {
												ms: pings[friend.join_address].ping ?? 0,
											})
										}}
									</template>
								</p>
							</div>
							<div class="flex items-center gap-1">
								<IconButton
									v-if="canViewFriendPack(friend)"
									type="quiet"
									:label="formatMessage(messages.viewPack)"
									@click="viewPack(friend)"
								>
									<PackageSearchIcon />
								</IconButton>
								<Button
									type="colored"
									color="brand"
									:disabled="joiningFriendId === friend.id"
									@click="playWithFriend(friend)"
								>
									<PlayIcon />
									{{ formatMessage(messages.playWith) }}
								</Button>
							</div>
						</li>
					</ul>
				</section>

				<section class="mx-auto mt-8 max-w-3xl">
					<h2 class="m-0 text-xs font-medium uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.sharedTitle) }}
					</h2>
					<p class="m-0 mt-1 text-sm text-secondary">
						{{ formatMessage(messages.sharedSubtitle) }}
					</p>

					<div
						v-if="sessionQuery.data.value"
						class="mt-3 flex flex-col gap-2 sm:flex-row sm:items-center"
					>
						<input
							v-model="shareName"
							type="text"
							class="min-h-9 min-w-0 flex-1 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm text-primary placeholder:text-secondary sm:max-w-[12rem]"
							:placeholder="formatMessage(messages.sharedNamePlaceholder)"
							:disabled="sharing"
						/>
						<input
							v-model="shareAddress"
							type="text"
							class="min-h-9 min-w-0 flex-1 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm text-primary placeholder:text-secondary"
							:placeholder="formatMessage(messages.sharedAddressPlaceholder)"
							:disabled="sharing"
							@keydown.enter.prevent="shareFromForm"
						/>
						<Button
							type="colored"
							color="brand"
							:disabled="sharing || !shareAddress.trim()"
							@click="shareFromForm"
						>
							<PlusIcon />
							{{ formatMessage(messages.sharedAdd) }}
						</Button>
					</div>

					<p
						v-if="!sessionQuery.data.value"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.sharedEmptySignedOut) }}
					</p>
					<p
						v-else-if="sharedServers.length === 0"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.sharedEmpty) }}
					</p>
					<ul v-else class="m-0 mt-2 flex list-none flex-col gap-1 p-0">
						<li
							v-for="server in sharedServers"
							:key="server.key"
							class="flex flex-wrap items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-3"
						>
							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<p class="m-0 truncate text-sm font-medium text-contrast">{{ server.name }}</p>
									<span
										v-if="isSharedByOther(server) && server.createdByNick"
										class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-secondary bg-surface-3"
									>
										{{
											formatMessage(messages.sharedBy, { name: server.createdByNick })
										}}
									</span>
									<span
										v-if="pings[server.address]"
										class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide"
										:class="
											pings[server.address].online
												? 'bg-highlight-green text-green'
												: 'bg-highlight-red text-red'
										"
									>
										{{
											pings[server.address].online
												? formatMessage(messages.online)
												: formatMessage(messages.offline)
										}}
									</span>
								</div>
								<p class="m-0 mt-0.5 font-mono text-xs text-secondary">{{ server.address }}</p>
								<div
									v-if="pings[server.address]?.online"
									class="mt-1 flex flex-wrap gap-3 text-[11px] text-secondary"
								>
									<span v-if="pings[server.address].ping != null">
										{{ formatMessage(messages.ping, { ms: pings[server.address].ping }) }}
									</span>
									<span
										v-if="
											pings[server.address].playersOnline != null &&
											pings[server.address].playersMax != null
										"
									>
										{{
											formatMessage(messages.players, {
												online: pings[server.address].playersOnline,
												max: pings[server.address].playersMax,
											})
										}}
									</span>
								</div>
							</div>
							<div class="flex items-center gap-1">
								<IconButton
									type="quiet"
									:label="formatMessage(messages.copy)"
									@click="copyAddress(server.address)"
								>
									<ClipboardCopyIcon />
								</IconButton>
								<IconButton
									type="quiet"
									:label="formatMessage(messages.savedDelete)"
									:disabled="joiningSavedKey === server.key"
									@click="requestDeleteShared(server)"
								>
									<TrashIcon />
								</IconButton>
								<Button
									type="colored"
									color="brand"
									:disabled="!resolvedInstanceId || joiningSavedKey === server.key"
									@click="playAddress(server.address, server.key)"
								>
									<PlayIcon />
									{{ formatMessage(messages.play) }}
								</Button>
							</div>
						</li>
					</ul>
				</section>

				<hr class="mx-auto mt-8 max-w-3xl border-0 border-t border-solid border-surface-5" />

				<section class="mx-auto mt-8 max-w-3xl">
					<h2 class="m-0 text-xs font-medium uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.localTitle) }}
					</h2>
					<p class="m-0 mt-1 text-xs text-secondary">
						{{ formatMessage(messages.localSubtitle) }}
					</p>

					<p
						v-if="!resolvedInstanceId"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.localEmptyNoInstance) }}
					</p>
					<p
						v-else-if="localWorldsQuery.isPending.value"
						class="m-0 mt-3 flex items-center gap-2 text-sm text-secondary"
					>
						<SpinnerIcon class="animate-spin" />
						{{ formatMessage(messages.loading) }}
					</p>
					<p
						v-else-if="localServers.length === 0"
						class="m-0 mt-3 text-sm text-secondary"
					>
						{{ formatMessage(messages.localEmpty) }}
					</p>
					<ul v-else class="m-0 mt-2 flex list-none flex-col gap-1 p-0">
						<li
							v-for="server in localServers"
							:key="server.key"
							class="flex flex-wrap items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-3"
						>
							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<p class="m-0 truncate text-sm font-medium text-contrast">{{ server.name }}</p>
									<span
										v-if="pings[server.address]"
										class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide"
										:class="
											pings[server.address].online
												? 'bg-highlight-green text-green'
												: 'bg-highlight-red text-red'
										"
									>
										{{
											pings[server.address].online
												? formatMessage(messages.online)
												: formatMessage(messages.offline)
										}}
									</span>
								</div>
								<p class="m-0 mt-0.5 font-mono text-xs text-secondary">{{ server.address }}</p>
								<div
									v-if="pings[server.address]?.online"
									class="mt-1 flex flex-wrap gap-3 text-[11px] text-secondary"
								>
									<span v-if="pings[server.address].ping != null">
										{{ formatMessage(messages.ping, { ms: pings[server.address].ping }) }}
									</span>
									<span
										v-if="
											pings[server.address].playersOnline != null &&
											pings[server.address].playersMax != null
										"
									>
										{{
											formatMessage(messages.players, {
												online: pings[server.address].playersOnline,
												max: pings[server.address].playersMax,
											})
										}}
									</span>
								</div>
							</div>
							<div class="flex items-center gap-1">
								<IconButton
									v-if="sessionQuery.data.value"
									type="quiet"
									:label="formatMessage(messages.shareLocal)"
									:disabled="sharing"
									@click="shareLocalServer(server)"
								>
									<ShareIcon />
								</IconButton>
								<IconButton
									type="quiet"
									:label="formatMessage(messages.copy)"
									@click="copyAddress(server.address)"
								>
									<ClipboardCopyIcon />
								</IconButton>
								<IconButton
									type="quiet"
									:label="formatMessage(messages.savedDelete)"
									:disabled="joiningSavedKey === server.key"
									@click="deleteLocalServer(server)"
								>
									<TrashIcon />
								</IconButton>
								<Button
									type="colored"
									color="brand"
									:disabled="!resolvedInstanceId || joiningSavedKey === server.key"
									@click="playAddress(server.address, server.key)"
								>
									<PlayIcon />
									{{ formatMessage(messages.play) }}
								</Button>
							</div>
						</li>
					</ul>
				</section>
			</template>
		</div>
	</div>
</template>
