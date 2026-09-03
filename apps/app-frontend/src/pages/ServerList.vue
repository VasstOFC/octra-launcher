<script setup lang="ts">
import {
	ClipboardCopyIcon,
	GlobeIcon,
	PlayIcon,
	RefreshCwIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import {
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, onActivated, ref } from 'vue'

import { toError } from '@/helpers/errors'
import { list as listInstances } from '@/helpers/instance'
import { listOctraServers } from '@/helpers/octra'
import { get_server_status, start_join_server } from '@/helpers/worlds'
import { instanceKeys } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

defineOptions({ name: 'ServerListPage' })

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()

const messages = defineMessages({
	title: {
		id: 'app.servers.title',
		defaultMessage: 'Servers',
	},
	subtitle: {
		id: 'app.servers.subtitle',
		defaultMessage: 'Your multiplayer server list, shared with Octra App.',
	},
	empty: {
		id: 'app.servers.empty',
		defaultMessage: 'No servers yet. Servers added in Octra App will show up here.',
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

type PingState = {
	online: boolean
	ping?: number
	playersOnline?: number
	playersMax?: number
	version?: string
}

const pinging = ref(false)
const pings = ref<Record<string, PingState>>({})

const serversQuery = useQuery({
	queryKey: ['octra', 'servers'],
	queryFn: listOctraServers,
})

const instancesQuery = useQuery({
	queryKey: instanceKeys.list(),
	queryFn: listInstances,
})

const servers = computed(() => serversQuery.data.value ?? [])

const launchInstanceId = computed(() => {
	const instances = instancesQuery.data.value ?? []
	if (instances.length === 0) return null
	const sorted = [...instances].sort((a, b) => {
		const aTime = a.last_played ? new Date(a.last_played).getTime() : 0
		const bTime = b.last_played ? new Date(b.last_played).getTime() : 0
		return bTime - aTime
	})
	return sorted[0]?.id ?? null
})

async function refreshPings() {
	if (servers.value.length === 0) return
	pinging.value = true
	try {
		await Promise.all(
			servers.value.map(async (server) => {
				try {
					const status = await get_server_status(server.address, null)
					pings.value = {
						...pings.value,
						[server.address]: {
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
						[server.address]: { online: false },
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

async function playServer(address: string) {
	const instanceId = launchInstanceId.value
	if (!instanceId) {
		addNotification({
			title: formatMessage(messages.noInstance),
			text: '',
			type: 'error',
		})
		return
	}
	try {
		await start_join_server(instanceId, address)
	} catch (error: unknown) {
		handleError(toError(error))
	}
}
</script>

<template>
	<div class="flex h-full min-h-0 flex-col">
		<div class="border-0 border-b border-solid border-bg-divider px-6 py-5">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div>
					<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>
					<p class="m-0 mt-1 text-sm text-secondary">{{ formatMessage(messages.subtitle) }}</p>
				</div>
				<Button :disabled="pinging || servers.length === 0" @click="refreshPings">
					<SpinnerIcon v-if="pinging" class="animate-spin" />
					<RefreshCwIcon v-else />
					{{ formatMessage(messages.refresh) }}
				</Button>
			</div>
		</div>
		<div class="min-h-0 flex-1 overflow-auto p-6">
			<div
				v-if="serversQuery.isPending.value"
				class="flex items-center justify-center gap-2 py-16 text-sm text-secondary"
			>
				<SpinnerIcon class="animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>
			<p v-else-if="servers.length === 0" class="m-0 mt-16 text-center text-sm text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<ul v-else class="m-0 mx-auto flex max-w-3xl list-none flex-col gap-2 p-0">
				<li
					v-for="server in servers"
					:key="`${server.address}-${server.name}`"
					class="flex flex-wrap items-center gap-3 rounded-2xl bg-bg-raised px-4 py-3"
				>
					<div class="min-w-0 flex-1">
						<div class="flex flex-wrap items-center gap-2">
							<p class="m-0 font-semibold text-contrast">{{ server.name }}</p>
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
							<span v-if="pings[server.address].version">{{ pings[server.address].version }}</span>
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
						<Button
							type="colored"
							color="brand"
							:disabled="!launchInstanceId"
							@click="playServer(server.address)"
						>
							<PlayIcon />
							{{ formatMessage(messages.play) }}
						</Button>
					</div>
				</li>
			</ul>
		</div>
	</div>
</template>
