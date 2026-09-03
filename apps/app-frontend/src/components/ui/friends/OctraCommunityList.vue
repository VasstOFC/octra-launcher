<script setup lang="ts">
import { MessageIcon, PlayIcon, SearchIcon } from '@modrinth/assets'
import type { ButtonMenuOption } from '@modrinth/ui'
import {
	Avatar,
	Button,
	ContextMenu,
	defineMessages,
	IconButton,
	Input,
	injectNotificationManager,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

import { useOctraCommunityAvatars } from '@/composables/use-octra-community-avatars'
import { handleSevereError } from '@/composables/use-error.js'
import { list } from '@/helpers/instance'
import { octraCommunity } from '@/helpers/octra-account.js'
import type { GameInstance } from '@/helpers/types'
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
	last_seen?: string | null
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
const { handleError } = injectNotificationManager()
const formatRelativeTime = useRelativeTime({ numeric: 'auto', style: 'short' })
const search = ref('')
const joiningId = ref<number | null>(null)
const memberOptions = useTemplateRef('memberOptions')

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
		instance_name: member.instance_name != null ? plainText(member.instance_name) : member.instance_name,
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
	return [
		{
			id: 'message-player',
			label: formatMessage(messages.messagePlayer),
			icon: MessageIcon,
			action: () => emit('messagePlayer', member.id),
		},
	]
}

function openMemberContextMenu(event: MouseEvent, member: OctraCommunityMember) {
	memberOptions.value?.open(event, createContextMenuOptions(member))
}

function canJoin(member: OctraCommunityMember) {
	return member.presence === 'ingame' && !!member.join_address?.trim()
}

function pickInstance(
	instances: GameInstance[],
	preferredName: string | null | undefined,
): GameInstance | null {
	if (instances.length === 0) return null
	const preferred = preferredName?.trim().toLowerCase()
	if (preferred) {
		const byName = instances.find((instance) => instance.name.toLowerCase() === preferred)
		if (byName) return byName
	}
	const sorted = [...instances].sort((a, b) => {
		const aTime = a.last_played ? new Date(a.last_played).getTime() : 0
		const bTime = b.last_played ? new Date(b.last_played).getTime() : 0
		return bTime - aTime
	})
	return sorted[0] ?? null
}

async function joinFriend(member: OctraCommunityMember) {
	const address = member.join_address?.trim()
	if (!address) return
	joiningId.value = member.id
	try {
		const instances = await list()
		const instance = pickInstance(instances, member.instance_name)
		if (!instance) {
			handleError(formatMessage(messages.noInstance))
			return
		}
		await start_join_server(instance.id, address)
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		joiningId.value = null
	}
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
	noInstance: {
		id: 'octra.community.no-instance',
		defaultMessage: 'Create or install an instance before joining a friend.',
	},
	messagePlayer: {
		id: 'octra.community.message-player',
		defaultMessage: 'Message player',
	},
	memberActionsLabel: {
		id: 'octra.community.actions.label',
		defaultMessage: 'Friend actions',
	},
})
</script>

<template>
	<div class="flex flex-col gap-3">
		<ContextMenu ref="memberOptions" :label="formatMessage(messages.memberActionsLabel)" />
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
			<p class="m-0 font-minecraft text-sm text-secondary">
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
					class="community-row grid grid-cols-[auto_1fr_auto] items-center gap-2 rounded-xl px-1 py-1.5 select-none"
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
						v-tooltip="formatMessage(messages.join)"
						type="standard"
						color="brand"
						:label="formatMessage(messages.join)"
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

