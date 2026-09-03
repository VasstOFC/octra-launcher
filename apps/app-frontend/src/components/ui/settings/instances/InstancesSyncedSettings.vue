<script setup lang="ts">
import { EditIcon, SaveIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Component } from 'vue'
import { computed, ref, shallowRef } from 'vue'

import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsRow,
} from '@/components/ui/settings/_shared'
import WorldItem from '@/components/ui/world/WorldItem.vue'
import {
	get_command_history,
	get_global_synced_options,
	type GlobalSyncedOptions,
	list_synced_servers,
	remove_synced_server,
	set_command_history,
	set_global_synced_option,
	type SyncedOption,
	type SyncedServer,
	update_synced_server,
} from '@/helpers/instance'
import {
	refreshServerData,
	refreshServers,
	type ServerData,
	type ServerWorld,
} from '@/helpers/worlds.ts'
import { instanceKeys, screenshotKeys } from '@/pages/instance/query-options'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.synced-options.title',
		defaultMessage: 'Sync',
	},
	panelDescription: {
		id: 'app.settings.synced-options.description',
		defaultMessage: 'Share servers, command history, and other options across your instances.',
	},
	syncOptionsGroup: {
		id: 'app.settings.synced-options.group',
		defaultMessage: 'Synced options',
	},
	multiplayerServers: {
		id: 'app.settings.synced-options.multiplayer-servers',
		defaultMessage: 'Multiplayer servers',
	},
	multiplayerServersDescription: {
		id: 'app.settings.synced-options.multiplayer-servers.description',
		defaultMessage: 'Sync multiplayer servers across your instances.',
	},
	commandHistory: {
		id: 'app.settings.synced-options.command-history',
		defaultMessage: 'Command history',
	},
	commandHistoryDescription: {
		id: 'app.settings.synced-options.command-history.description',
		defaultMessage: 'Sync command history across your instances.',
	},
	creativeHotbars: {
		id: 'app.settings.synced-options.creative-hotbars',
		defaultMessage: 'Saved creative hotbars',
	},
	creativeHotbarsDescription: {
		id: 'app.settings.synced-options.creative-hotbars.description',
		defaultMessage: 'Sync saved creative hotbars across your instances.',
	},
	screenshots: {
		id: 'app.settings.synced-options.screenshots',
		defaultMessage: 'Screenshots',
	},
	screenshotsDescription: {
		id: 'app.settings.synced-options.screenshots.description',
		defaultMessage: 'View screenshots from your instances in one place.',
	},
	commandHistoryEditorTitle: {
		id: 'app.settings.synced-options.command-history.editor-title',
		defaultMessage: 'Edit command history',
	},
	serverEditorTitle: {
		id: 'app.settings.synced-options.multiplayer-servers.editor-title',
		defaultMessage: 'Edit synced servers',
	},
	editServerTitle: {
		id: 'instance.edit-server.title',
		defaultMessage: 'Edit server',
	},
	serverName: {
		id: 'app.settings.synced-options.multiplayer-servers.name',
		defaultMessage: 'Server name',
	},
	serverAddress: {
		id: 'app.settings.synced-options.multiplayer-servers.address',
		defaultMessage: 'Server address',
	},
	noSyncedServers: {
		id: 'app.settings.synced-options.multiplayer-servers.empty',
		defaultMessage: 'No user-added servers are currently synced.',
	},
	noServersSyncedYet: {
		id: 'app.settings.synced-options.multiplayer-servers.none-synced-yet',
		defaultMessage: 'No servers synced yet',
	},
})

const globalRows: Array<{
	option: SyncedOption
	title: keyof typeof messages
	description?: keyof typeof messages
	editable?: 'servers' | 'commands'
}> = [
	{
		option: 'multiplayer_servers',
		title: 'multiplayerServers',
		description: 'multiplayerServersDescription',
		editable: 'servers',
	},
	{
		option: 'command_history',
		title: 'commandHistory',
		description: 'commandHistoryDescription',
		editable: 'commands',
	},
	{
		option: 'creative_hotbars',
		title: 'creativeHotbars',
		description: 'creativeHotbarsDescription',
	},
	{
		option: 'screenshots',
		title: 'screenshots',
		description: 'screenshotsDescription',
	},
]

const globalSyncedOptionsQueryKey = ['global-synced-options'] as const
const globalSyncedOptionsMutationKey = ['global-synced-options', 'set'] as const
const defaultGlobalOptions: GlobalSyncedOptions = {
	command_history: false,
	multiplayer_servers: false,
	creative_hotbars: false,
	screenshots: false,
}

const globalOptionsQuery = useQuery({
	queryKey: globalSyncedOptionsQueryKey,
	queryFn: get_global_synced_options,
})
const globalOptions = computed(() => globalOptionsQuery.data.value ?? defaultGlobalOptions)
const commandHistoryModal = ref<InstanceType<typeof NewModal> | null>(null)
const serverEditorModal = ref<InstanceType<typeof NewModal> | null>(null)
const editServerModal = ref<InstanceType<typeof NewModal> | null>(null)
const commandHistory = ref('')
const syncedServers = ref<SyncedServer[]>(
	(await list_synced_servers().catch((error) => {
		handleError(error)
		return []
	})) ?? [],
)
const editedServer = ref<SyncedServer | null>(null)
const serverData = ref<Record<string, ServerData>>({})
const editorComponent = shallowRef<Component | null>(null)

const syncedServerCards = computed(() =>
	syncedServers.value.map((server, index) => ({
		server,
		world: {
			name: server.name,
			type: 'server',
			index,
			server_id: server.id,
			address: server.address,
			pack_status:
				server.accept_textures === true
					? 'enabled'
					: server.accept_textures === false
						? 'disabled'
						: 'prompt',
			display_status: 'normal',
		} satisfies ServerWorld,
	})),
)

async function invalidateSyncedOptions() {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
		queryClient.invalidateQueries({ queryKey: globalSyncedOptionsQueryKey }),
		queryClient.invalidateQueries({ queryKey: screenshotKeys.all }),
	])
}

type GlobalOptionMutationVariables = {
	option: SyncedOption
	enabled: boolean
}

const globalOptionMutation = useMutation({
	mutationKey: globalSyncedOptionsMutationKey,
	mutationFn: ({ option, enabled }: GlobalOptionMutationVariables) =>
		set_global_synced_option(option, enabled),
	onMutate: async ({ option, enabled }) => {
		await queryClient.cancelQueries({ queryKey: globalSyncedOptionsQueryKey })
		const previous = globalOptions.value[option]

		queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: enabled,
		}))

		return { previous }
	},
	onError: (error, { option }, context) => {
		queryClient.setQueryData<GlobalSyncedOptions>(globalSyncedOptionsQueryKey, (current) => ({
			...(current ?? defaultGlobalOptions),
			[option]: context?.previous ?? defaultGlobalOptions[option],
		}))
		handleError(error)
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: globalSyncedOptionsMutationKey }) === 1) {
			await invalidateSyncedOptions()
		}
	},
})

function toggleGlobalOption(option: SyncedOption, enabled: boolean) {
	globalOptionMutation.mutate({ option, enabled })
}

async function openCommandHistoryEditor() {
	commandHistory.value = await get_command_history().catch((error) => {
		handleError(error)
		return ''
	})
	if (!editorComponent.value) {
		const [editor] = await Promise.all([
			import('vue3-ace-editor'),
			import('@modrinth/ui/src/utils/ace-theme'),
			import('@modrinth/ui/src/utils/ace-mode-mcfunction'),
		])
		editorComponent.value = editor.VAceEditor
	}
	commandHistoryModal.value?.show()
}

async function saveCommandHistory() {
	try {
		commandHistory.value = await set_command_history(commandHistory.value)
		commandHistoryModal.value?.hide()
	} catch (error) {
		handleError(error)
	}
}

async function openServerEditor() {
	syncedServers.value = await list_synced_servers().catch((error) => {
		handleError(error)
		return []
	})
	serverData.value = {}
	serverEditorModal.value?.show()
	await refreshServers(
		syncedServerCards.value.map(({ world }) => world),
		serverData.value,
		null,
	)
}

function openSyncedServerEditor(server: SyncedServer) {
	editedServer.value = { ...server }
	editServerModal.value?.show()
}

async function saveSyncedServer() {
	if (!editedServer.value) return
	const server = editedServer.value

	try {
		await update_synced_server(server)
		const index = syncedServers.value.findIndex(({ id }) => id === server.id)
		if (index !== -1) {
			syncedServers.value[index] = { ...server }
		}
		editServerModal.value?.hide()
		serverData.value[server.address] = { refreshing: true }
		await refreshServerData(serverData.value[server.address], null, server.address)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	} catch (error) {
		handleError(error)
	}
}

async function refreshSyncedServer(address: string) {
	serverData.value[address] ??= { refreshing: true }
	await refreshServerData(serverData.value[address], null, address)
}

async function removeSyncedServer(serverId: string) {
	try {
		await remove_synced_server(serverId)
		syncedServers.value = syncedServers.value.filter((server) => server.id !== serverId)
		await queryClient.invalidateQueries({ queryKey: ['worlds'] })
	} catch (error) {
		handleError(error)
	}
}
</script>

<template>
	<div>
		<NewModal
			ref="commandHistoryModal"
			:header="formatMessage(messages.commandHistoryEditorTitle)"
			class="command-history-modal"
			max-width="700px"
			width="700px"
		>
			<component
				:is="editorComponent"
				v-if="editorComponent"
				v-model:value="commandHistory"
				lang="mcfunction"
				theme="modrinth"
				:print-margin="false"
				class="command-history-editor ace-modrinth rounded-[20px] !border !border-solid !border-surface-5"
				style="height: 420px; font-size: 0.875rem"
			/>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="commandHistoryModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button type="colored" color="brand" @click="saveCommandHistory">
						<SaveIcon />
						{{ formatMessage(commonMessages.saveButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<NewModal
			ref="serverEditorModal"
			:header="formatMessage(messages.serverEditorTitle)"
			scrollable
			actions-divider
			no-padding
			max-content-height="34.5rem"
			max-width="750px"
			width="750px"
		>
			<p v-if="syncedServers.length === 0" class="m-0 px-6 py-4 text-secondary">
				{{ formatMessage(messages.noSyncedServers) }}
			</p>
			<div v-else class="flex flex-col gap-2 px-6 py-4">
				<WorldItem
					v-for="{ server, world } in syncedServerCards"
					:key="server.id"
					:world="world"
					card-background="surface-2"
					:show-play-button="false"
					:refreshing="serverData[server.address]?.refreshing"
					:server-status="serverData[server.address]?.status"
					:rendered-motd="serverData[server.address]?.renderedMotd"
					@refresh="refreshSyncedServer(server.address)"
					@edit="openSyncedServerEditor(server)"
					@delete="removeSyncedServer(server.id)"
				/>
			</div>
			<template #actions>
				<div class="flex justify-end">
					<Button type="outlined" @click="serverEditorModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.closeButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<NewModal
			ref="editServerModal"
			:header="formatMessage(messages.editServerTitle)"
			max-width="500px"
			width="500px"
		>
			<div v-if="editedServer" class="flex flex-col gap-4">
				<label class="flex flex-col gap-2 font-semibold text-contrast">
					{{ formatMessage(messages.serverName) }}
					<Input v-model="editedServer.name" autocomplete="off" wrapper-class="w-full" />
				</label>
				<label class="flex flex-col gap-2 font-semibold text-contrast">
					{{ formatMessage(messages.serverAddress) }}
					<Input v-model="editedServer.address" autocomplete="off" wrapper-class="w-full" />
				</label>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="editServerModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!editedServer?.address"
						@click="saveSyncedServer"
					>
						<SaveIcon />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup :label="formatMessage(messages.syncOptionsGroup)">
			<SettingsRow
				v-for="row in globalRows"
				:key="row.option"
				:control-id="`global-sync-${row.option}`"
				:title="formatMessage(messages[row.title])"
				:description="row.description ? formatMessage(messages[row.description]) : undefined"
			>
				<template #default="{ labelledBy, controlId }">
					<div class="flex shrink-0 items-center gap-2">
						<span
							v-if="row.editable"
							v-tooltip="
								row.editable === 'servers' && syncedServers.length === 0
									? formatMessage(messages.noServersSyncedYet)
									: formatMessage(commonMessages.editButton)
							"
							class="flex"
						>
							<IconButton
								type="outlined"
								circular
								:disabled="
									!globalOptions[row.option] ||
									(row.editable === 'servers' && syncedServers.length === 0)
								"
								:label="formatMessage(commonMessages.editButton)"
								@click="
									row.editable === 'commands' ? openCommandHistoryEditor() : openServerEditor()
								"
							>
								<EditIcon />
							</IconButton>
						</span>
						<Toggle
							:id="controlId"
							:model-value="globalOptions[row.option]"
							:aria-labelledby="labelledBy"
							@update:model-value="(enabled) => toggleGlobalOption(row.option, enabled)"
						/>
					</div>
				</template>
			</SettingsRow>
		</SettingsGroup>
	</div>
</template>

<style>
.command-history-editor.ace-modrinth {
	background-color: var(--surface-2);
}

.command-history-editor.ace-modrinth .ace_gutter {
	background: var(--surface-1);
}

.command-history-editor.ace-modrinth .ace_marker-layer .ace_active-line {
	background: var(--surface-2-5);
}

.command-history-editor.ace-modrinth .ace_gutter-active-line {
	background-color: var(--surface-1-5);
}

.command-history-editor.ace-modrinth.ace_multiselect .ace_selection.ace_start {
	box-shadow: 0 0 3px 0 var(--surface-2);
}

.command-history-modal > [data-modal-content] {
	padding-bottom: 0;
}
</style>
