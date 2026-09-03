<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	Input,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

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
	last_seen?: string | null
}

const props = defineProps<{
	session: OctraAccountSession | null
	loadingSession?: boolean
}>()

const emit = defineEmits<{
	signIn: []
	register: []
}>()

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ numeric: 'auto', style: 'short' })
const search = ref('')

const query = useQuery({
	queryKey: computed(() => ['octra-community', props.session?.username ?? null]),
	queryFn: () => octraCommunity(),
	enabled: computed(() => !!props.session),
	staleTime: 10_000,
	refetchInterval: 15_000,
})

const snapshot = computed(() => query.data.value)
const connected = computed(() => !!props.session && !!snapshot.value?.connected)
const members = computed<OctraCommunityMember[]>(() => snapshot.value?.members ?? [])
const listLoading = computed(
	() => !!props.loadingSession || (!!props.session && query.isLoading.value),
)

const filtered = computed(() => {
	const q = search.value.trim().toLowerCase()
	const list = members.value.slice().sort((a, b) => {
		const rank = (presence?: string) => {
			if (presence === 'ingame') return 0
			if (presence === 'launcher') return 1
			return 2
		}
		const byPresence = rank(a.presence) - rank(b.presence)
		if (byPresence !== 0) return byPresence
		return a.minecraft_nick.localeCompare(b.minecraft_nick, undefined, { sensitivity: 'base' })
	})
	if (!q) return list
	return list.filter((member) => member.minecraft_nick.toLowerCase().includes(q))
})

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

const messages = defineMessages({
	heading: {
		id: 'octra.community.heading',
		defaultMessage: 'Friends',
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
		defaultMessage: 'Playing {name}',
	},
	inGameUnknown: {
		id: 'octra.community.in-game-unknown',
		defaultMessage: 'In game',
	},
	lastSeen: {
		id: 'octra.community.last-seen',
		defaultMessage: 'Last seen {time}',
	},
})
</script>

<template>
	<div class="flex flex-col gap-3">
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
							connected
								? formatMessage(messages.connected)
								: formatMessage(messages.disconnected)
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

		<template v-else-if="members.length === 0">
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
			<div class="flex flex-col gap-0.5">
				<div
					v-for="member in filtered"
					:key="member.id"
					class="grid grid-cols-[auto_1fr] items-center gap-2 rounded-xl px-1 py-1.5 select-none"
				>
					<div class="relative shrink-0">
						<Avatar :src="member.avatar_url" size="32px" circle />
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
				</div>
			</div>
			<p v-if="filtered.length === 0 && search" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.noMatch, { query: search }) }}
			</p>
		</template>
	</div>
</template>
