<script setup lang="ts">
import { DownloadIcon, PlayIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'
import { useRouter } from 'vue-router'

import {
	canViewFriendPack,
	type PlayWithFriendMember,
} from '@/composables/play-with-friend'
import { handleSevereError } from '@/composables/use-error.js'
import { get_project, get_project_versions } from '@/helpers/cache.js'
import {
	install_create_modpack_instance,
	installJobInstanceId,
	wait_for_install_job,
} from '@/helpers/install'
import { getInstanceIconUrl, list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { start_join_server } from '@/helpers/worlds'
import { injectAppEvents } from '@/providers/app-events'

const emit = defineEmits<{
	opened: [memberId: number]
	closed: []
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()
const router = useRouter()
const appEvents = injectAppEvents()

const playModal = useTemplateRef<InstanceType<typeof NewModal>>('playModal')
const playMember = ref<PlayWithFriendMember | null>(null)
const playStep = ref<'choose' | 'own'>('choose')
const playInstances = ref<GameInstance[]>([])
const selectedInstanceId = ref<string | null>(null)
const playBusy = ref(false)
const playError = ref('')
const activeMemberId = ref<number | null>(null)

const messages = defineMessages({
	playWithToast: {
		id: 'octra.community.play-with-toast',
		defaultMessage: 'Joining {nick}…',
	},
	playWithTitle: {
		id: 'octra.community.play-with-title',
		defaultMessage: 'Play with {nick}',
	},
	playWithHint: {
		id: 'octra.community.play-with-hint',
		defaultMessage: 'Join with a pack you already have, or download the one {nick} is using.',
	},
	playOwnPack: {
		id: 'octra.community.play-own-pack',
		defaultMessage: 'Use my pack',
	},
	playOwnPackHint: {
		id: 'octra.community.play-own-pack-hint',
		defaultMessage: 'Pick one of your installed instances and join the same server.',
	},
	playFriendPack: {
		id: 'octra.community.play-friend-pack',
		defaultMessage: "Download friend's pack",
	},
	playFriendPackHint: {
		id: 'octra.community.play-friend-pack-hint',
		defaultMessage: 'Install {name} if you don’t have it yet, then join.',
	},
	playFriendPackUnknown: {
		id: 'octra.community.play-friend-pack-unknown',
		defaultMessage: 'their pack',
	},
	playPickInstance: {
		id: 'octra.community.play-pick-instance',
		defaultMessage: 'Choose an instance',
	},
	playJoin: {
		id: 'octra.community.play-join',
		defaultMessage: 'Join',
	},
	playBack: {
		id: 'octra.community.play-back',
		defaultMessage: 'Back',
	},
	playCancel: {
		id: 'octra.community.play-cancel',
		defaultMessage: 'Cancel',
	},
	playBusy: {
		id: 'octra.community.play-busy',
		defaultMessage: 'Installing pack and joining…',
	},
	friendPackMissing: {
		id: 'octra.community.friend-pack-missing',
		defaultMessage: "Couldn't find that pack online. Pick one of your instances instead.",
	},
	friendPackInstallFailed: {
		id: 'octra.community.friend-pack-install-failed',
		defaultMessage: 'Pack installed, but the new instance could not be opened.',
	},
	noInstance: {
		id: 'octra.community.no-instance',
		defaultMessage: 'Create or install an instance before joining a friend.',
	},
})

function isPlayable(instance: GameInstance) {
	return instance.install_stage === 'installed' || instance.install_stage === 'pack_installed'
}

function sortInstances(instances: GameInstance[]) {
	return [...instances].sort((a, b) => {
		const aTime = a.last_played ? new Date(a.last_played).getTime() : 0
		const bTime = b.last_played ? new Date(b.last_played).getTime() : 0
		return bTime - aTime
	})
}

function packIdsFromInstance(instance: GameInstance): {
	projectId?: string
	versionId?: string
} {
	const link = instance.link
	if (!link) return {}
	if (link.type === 'modrinth_modpack') {
		return { projectId: link.project_id, versionId: link.version_id }
	}
	if (link.type === 'imported_modpack') {
		return {
			projectId: link.project_id ?? undefined,
			versionId: link.version_id ?? undefined,
		}
	}
	if (link.type === 'shared_instance') {
		return {
			projectId: link.modpack_project_id ?? undefined,
			versionId: link.modpack_version_id ?? undefined,
		}
	}
	if (link.type === 'server_project_modpack') {
		return {
			projectId: link.content_project_id ?? link.project_id,
			versionId: link.content_version_id ?? link.version_id,
		}
	}
	return {
		projectId: 'project_id' in link ? (link.project_id ?? undefined) : undefined,
		versionId: 'version_id' in link ? (link.version_id ?? undefined) : undefined,
	}
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
	return sortInstances(instances)[0] ?? null
}

function matchFriendInstance(
	instances: GameInstance[],
	member: PlayWithFriendMember,
): GameInstance | null {
	const playable = sortInstances(instances.filter(isPlayable))
	const projectId = member.pack_project_id?.trim()
	if (projectId) {
		const byPack = playable.find(
			(instance) => packIdsFromInstance(instance).projectId === projectId,
		)
		if (byPack) return byPack
	}
	return pickInstance(playable, member.instance_name)
}

function onPlayModalHide() {
	activeMemberId.value = null
	playStep.value = 'choose'
	playError.value = ''
	emit('closed')
}

async function loadPlayInstances() {
	const instances = await list()
	playInstances.value = sortInstances(instances.filter(isPlayable))
	if (
		!selectedInstanceId.value ||
		!playInstances.value.some((instance) => instance.id === selectedInstanceId.value)
	) {
		selectedInstanceId.value = playInstances.value[0]?.id ?? null
	}
}

async function open(member: PlayWithFriendMember) {
	playMember.value = member
	playStep.value = 'choose'
	playError.value = ''
	playBusy.value = false
	activeMemberId.value = member.id
	emit('opened', member.id)
	try {
		await loadPlayInstances()
		playModal.value?.show()
	} catch (error) {
		activeMemberId.value = null
		emit('closed')
		handleSevereError(error, handleError)
	}
}

async function joinWithInstance(instance: GameInstance, member: PlayWithFriendMember) {
	const address = member.join_address?.trim()
	if (!address) return
	await start_join_server(instance.id, address)
	addNotification({
		title: formatMessage(messages.playWithToast, { nick: member.minecraft_nick }),
		text: '',
		type: 'success',
	})
	playModal.value?.hide()
}

async function confirmOwnPack() {
	const member = playMember.value
	const instance = playInstances.value.find((item) => item.id === selectedInstanceId.value)
	if (!member || !instance) {
		playError.value = formatMessage(messages.noInstance)
		return
	}
	playBusy.value = true
	playError.value = ''
	try {
		await joinWithInstance(instance, member)
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		playBusy.value = false
	}
}

async function latestVersionId(projectId: string, preferredVersionId?: string | null) {
	if (preferredVersionId?.trim()) return preferredVersionId.trim()
	const versions = (await get_project_versions(projectId)) as Array<{ id?: string }> | null
	const first = versions?.find((version) => typeof version.id === 'string')
	return first?.id ?? null
}

async function installFriendPackAndJoin(
	member: PlayWithFriendMember,
	projectId: string,
	versionId: string,
	title: string,
	iconUrl?: string | null,
) {
	const job = await install_create_modpack_instance({
		type: 'fromVersionId',
		project_id: projectId,
		version_id: versionId,
		title,
		icon_url: iconUrl,
	})
	const finished = await wait_for_install_job(appEvents, job.job_id)
	const instanceId = installJobInstanceId(finished)
	if (!instanceId) {
		throw new Error(formatMessage(messages.friendPackInstallFailed))
	}
	await start_join_server(instanceId, member.join_address!.trim())
	addNotification({
		title: formatMessage(messages.playWithToast, { nick: member.minecraft_nick }),
		text: '',
		type: 'success',
	})
	playModal.value?.hide()
}

async function useFriendPack() {
	const member = playMember.value
	if (!member) return
	playBusy.value = true
	playError.value = ''
	try {
		await loadPlayInstances()
		const existing = matchFriendInstance(playInstances.value, member)
		if (existing) {
			await joinWithInstance(existing, member)
			return
		}

		let projectId = member.pack_project_id?.trim() || ''
		let versionId = member.pack_version_id?.trim() || ''
		let title = member.instance_name?.trim() || member.minecraft_nick
		let iconUrl: string | null = null

		if (!projectId && member.instance_name?.trim()) {
			const results = await labrinth.projects_v2.search({
				query: member.instance_name.trim(),
				facets: [['project_type:modpack']],
				limit: 8,
			})
			const wanted = member.instance_name.trim().toLowerCase()
			const hit =
				results.hits.find((item) => item.title.toLowerCase() === wanted) ??
				results.hits.find((item) => item.slug.toLowerCase() === wanted) ??
				results.hits[0]
			if (hit) {
				projectId = hit.project_id
				title = hit.title
				iconUrl = hit.icon_url || null
			}
		}

		if (projectId && !versionId) {
			versionId = (await latestVersionId(projectId)) ?? ''
		}

		if (!projectId || !versionId) {
			playError.value = formatMessage(messages.friendPackMissing)
			playStep.value = 'own'
			await loadPlayInstances()
			return
		}

		if (!iconUrl) {
			const project = (await get_project(projectId).catch(() => null)) as {
				title?: string
				icon_url?: string
			} | null
			title = project?.title || title
			iconUrl = project?.icon_url ?? null
		}

		await installFriendPackAndJoin(member, projectId, versionId, title, iconUrl)
	} catch (error) {
		handleSevereError(error, handleError)
	} finally {
		playBusy.value = false
	}
}

async function viewPack(member: PlayWithFriendMember) {
	if (!canViewFriendPack(member)) return
	const projectId = member.pack_project_id?.trim()
	if (projectId) {
		await router.push(`/project/${projectId}`)
		return
	}

	const instanceName = member.instance_name?.trim()
	if (!instanceName) return

	try {
		const results = await labrinth.projects_v2.search({
			query: instanceName,
			facets: [['project_type:modpack']],
			limit: 8,
		})
		const wanted = instanceName.toLowerCase()
		const hit =
			results.hits.find((item) => item.title.toLowerCase() === wanted) ??
			results.hits.find((item) => item.slug.toLowerCase() === wanted)
		if (hit) {
			await router.push(`/project/${hit.project_id ?? hit.slug}`)
			return
		}
		await router.push({
			path: '/browse/modpack',
			query: { q: instanceName },
		})
	} catch (error) {
		handleSevereError(error, handleError)
	}
}

defineExpose({
	open,
	viewPack,
	activeMemberId,
})
</script>

<template>
	<NewModal
		ref="playModal"
		:header="formatMessage(messages.playWithTitle, { nick: playMember?.minecraft_nick || '…' })"
		max-width="460px"
		:on-hide="onPlayModalHide"
	>
		<div class="flex flex-col gap-3">
			<p class="m-0 text-sm text-secondary">
				{{
					formatMessage(messages.playWithHint, {
						nick: playMember?.minecraft_nick || '',
					})
				}}
			</p>
			<p v-if="playError" class="m-0 text-sm text-red">{{ playError }}</p>
			<p v-if="playBusy" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.playBusy) }}
			</p>
			<template v-if="playStep === 'choose'">
				<button
					type="button"
					class="flex items-start gap-3 rounded-lg border border-solid border-surface-5 bg-button-bg px-3 py-3 text-left text-primary hover:border-brand"
					:disabled="playBusy"
					@click="playStep = 'own'"
				>
					<PlayIcon class="mt-0.5 size-5 shrink-0 text-brand" />
					<span class="min-w-0">
						<span class="block font-medium text-contrast">
							{{ formatMessage(messages.playOwnPack) }}
						</span>
						<span class="mt-0.5 block text-xs text-secondary">
							{{ formatMessage(messages.playOwnPackHint) }}
						</span>
					</span>
				</button>
				<button
					type="button"
					class="flex items-start gap-3 rounded-lg border border-solid border-surface-5 bg-button-bg px-3 py-3 text-left text-primary hover:border-brand"
					:disabled="playBusy"
					@click="useFriendPack"
				>
					<DownloadIcon class="mt-0.5 size-5 shrink-0 text-brand" />
					<span class="min-w-0">
						<span class="block font-medium text-contrast">
							{{ formatMessage(messages.playFriendPack) }}
						</span>
						<span class="mt-0.5 block text-xs text-secondary">
							{{
								formatMessage(messages.playFriendPackHint, {
									name:
										playMember?.instance_name ||
										formatMessage(messages.playFriendPackUnknown),
								})
							}}
						</span>
					</span>
				</button>
				<div class="flex justify-end">
					<Button :disabled="playBusy" @click="playModal?.hide()">
						{{ formatMessage(messages.playCancel) }}
					</Button>
				</div>
			</template>
			<template v-else>
				<p class="m-0 text-xs font-medium text-secondary">
					{{ formatMessage(messages.playPickInstance) }}
				</p>
				<div class="flex max-h-64 flex-col gap-1 overflow-y-auto">
					<button
						v-for="instance in playInstances"
						:key="instance.id"
						type="button"
						class="flex items-center gap-2 rounded-lg border border-solid px-2 py-2 text-left"
						:class="
							selectedInstanceId === instance.id
								? 'border-brand bg-brand/10'
								: 'border-surface-5 bg-button-bg'
						"
						:disabled="playBusy"
						@click="selectedInstanceId = instance.id"
					>
						<img
							v-if="getInstanceIconUrl(instance.icon_path)"
							class="size-8 shrink-0 rounded-md object-cover"
							:src="getInstanceIconUrl(instance.icon_path)!"
							alt=""
						/>
						<span
							v-else
							class="flex size-8 shrink-0 items-center justify-center rounded-md bg-surface-3 text-xs text-secondary"
						>
							{{ instance.name.slice(0, 1).toUpperCase() }}
						</span>
						<span class="min-w-0">
							<span class="block truncate text-sm text-contrast">{{ instance.name }}</span>
							<span class="block truncate text-[11px] text-secondary">
								{{ instance.game_version }} · {{ instance.loader }}
							</span>
						</span>
					</button>
				</div>
				<p v-if="playInstances.length === 0" class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.noInstance) }}
				</p>
				<div class="flex justify-end gap-2">
					<Button :disabled="playBusy" @click="playStep = 'choose'">
						{{ formatMessage(messages.playBack) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="playBusy || !selectedInstanceId"
						@click="confirmOwnPack"
					>
						{{ formatMessage(messages.playJoin) }}
					</Button>
				</div>
			</template>
		</div>
	</NewModal>
</template>
