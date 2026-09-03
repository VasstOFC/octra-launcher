<script setup lang="ts">
import {
	ClipboardCopyIcon,
	GlobeIcon,
	PackageSearchIcon,
	PlayIcon,
	RefreshCwIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onActivated, ref, useTemplateRef, watch } from 'vue'

import PlayWithFriendModal from '@/components/ui/friends/PlayWithFriendModal.vue'
import {
	canPlayWithFriend,
	canViewFriendPack,
	type PlayWithFriendMember,
} from '@/composables/play-with-friend'
import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { toError } from '@/helpers/errors'
import { list as listInstances } from '@/helpers/instance'
import { listOctraServers, removeOctraServer } from '@/helpers/octra'
import {
	octraAccountSession,
	octraCommunity,
	octraSharedServersDelete,
	octraSharedServersList,
} from '@/helpers/octra-account.js'
import { get_server_status, start_join_server } from '@/helpers/worlds'
import { instanceKeys } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

defineOptions({ name: 'ServerListPage' })

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

type SavedServer = {
	key: string
	name: string
	address: string
	source: 'local' | 'shared'
	sharedId?: number
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
	savedTitle: {
		id: 'app.servers.saved.title',
		defaultMessage: 'Saved servers',
	},
	savedEmpty: {
		id: 'app.servers.saved.empty',
		defaultMessage: 'No saved servers yet. Add shared ones from the friends panel Servers tab.',
	},
	savedSharedBadge: {
		id: 'app.servers.saved.shared-badge',
		defaultMessage: 'Shared',
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
const quickAddress = ref('')
const selectedInstanceId = ref<string | null>(null)
const joiningQuick = ref(false)
const joiningFriendId = ref<number | null>(null)
const joiningSavedKey = ref<string | null>(null)
const pinging = ref(false)
const pings = ref<Record<string, PingState>>({})

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

const localServersQuery = useQuery({
	queryKey: ['octra', 'servers'],
	queryFn: listOctraServers,
})

const sharedServersQuery = useQuery({
	queryKey: computed(() => ['octra', 'shared-servers', sessionQuery.data.value?.username ?? null]),
	queryFn: () => octraSharedServersList(),
	enabled: computed(() => !!sessionQuery.data.value),
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

const savedServers = computed<SavedServer[]>(() => {
	const byAddress = new Map<string, SavedServer>()
	for (const server of localServersQuery.data.value ?? []) {
		const address = server.address.trim()
		if (!address) continue
		const key = address.toLowerCase()
		byAddress.set(key, {
			key: `local:${key}`,
			name: server.name || address,
			address,
			source: 'local',
		})
	}
	for (const server of sharedServersQuery.data.value ?? []) {
		const address = String(server.address ?? '').trim()
		if (!address) continue
		const key = address.toLowerCase()
		if (byAddress.has(key)) continue
		byAddress.set(key, {
			key: `shared:${server.id}`,
			name: server.name || address,
			address,
			source: 'shared',
			sharedId: Number(server.id),
		})
	}
	return [...byAddress.values()].sort((a, b) =>
		a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
	)
})

const pageLoading = computed(
	() =>
		localServersQuery.isPending.value ||
		instancesQuery.isPending.value ||
		(!!sessionQuery.data.value && communityQuery.isPending.value),
)

async function refreshPings() {
	const addresses = [
		...new Set([
			...savedServers.value.map((server) => server.address),
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

async function deleteSavedServer(server: SavedServer) {
	joiningSavedKey.value = server.key
	try {
		if (server.source === 'shared' && server.sharedId != null) {
			await octraSharedServersDelete(server.sharedId)
			await queryClient.invalidateQueries({ queryKey: ['octra', 'shared-servers'] })
		} else {
			await removeOctraServer(server.address)
			await queryClient.invalidateQueries({ queryKey: ['octra', 'servers'] })
		}
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

		<div class="border-0 border-b border-solid border-surface-5 px-6 py-5">
			<div class="flex flex-wrap items-start justify-between gap-3">
				<div class="min-w-0">
					<h1 class="m-0 text-2xl font-semibold text-contrast">
						{{ formatMessage(messages.title) }}
					</h1>
					<p class="m-0 mt-1 text-sm text-secondary">{{ formatMessage(messages.subtitle) }}</p>
				</div>
				<Button
					:disabled="pinging || (savedServers.length === 0 && friendsInGame.length === 0)"
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
						{{ formatMessage(messages.savedTitle) }}
					</h2>
					<p v-if="savedServers.length === 0" class="m-0 mt-3 text-sm text-secondary">
						{{ formatMessage(messages.savedEmpty) }}
					</p>
					<ul v-else class="m-0 mt-2 flex list-none flex-col gap-1 p-0">
						<li
							v-for="server in savedServers"
							:key="server.key"
							class="flex flex-wrap items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-3"
						>
							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<p class="m-0 truncate text-sm font-medium text-contrast">{{ server.name }}</p>
									<span
										v-if="server.source === 'shared'"
										class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-secondary bg-surface-3"
									>
										{{ formatMessage(messages.savedSharedBadge) }}
									</span>
									<span
										v-if="pings[server.address]"
										class="rounded-full px-2 py-0.5 text-[10px] font-bold"
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
									@click="deleteSavedServer(server)"
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
