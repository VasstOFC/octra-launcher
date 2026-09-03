<script setup lang="ts">
import { BoxIcon, FolderOpenIcon, FolderSearchIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { ref, watch } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsRow,
	SettingsStack,
} from '@/components/ui/settings/_shared'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { purge_cache_types } from '@/helpers/cache.js'
import { get, set } from '@/helpers/settings.ts'
import { showAppDbBackupsFolder } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const appSettings = useAppSettings()
const settings = ref(await get())
const purgeCacheConfirmModal = ref(null)
const alwaysShowCopyDetailsFlag = 'always_show_copy_details'

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.resource-management.title',
		defaultMessage: 'Storage',
	},
	panelDescription: {
		id: 'app.settings.resource-management.description',
		defaultMessage: 'Manage where Octra stores files, cache, and download concurrency.',
	},
	locationGroup: {
		id: 'app.settings.resource-management.location.group',
		defaultMessage: 'Location',
	},
	performanceGroup: {
		id: 'app.settings.resource-management.performance.group',
		defaultMessage: 'Downloads and writes',
	},
	maintenanceGroup: {
		id: 'app.settings.resource-management.maintenance.group',
		defaultMessage: 'Maintenance',
	},
	appDirectoryTitle: {
		id: 'app.settings.resource-management.app-directory.title',
		defaultMessage: 'App directory',
	},
	appDirectoryDescription: {
		id: 'app.settings.resource-management.app-directory.description',
		defaultMessage:
			'Where Octra App stores instances and other files. Changes take effect after restarting the app.',
	},
	selectAppDirectory: {
		id: 'app.settings.resource-management.app-directory.select',
		defaultMessage: 'Select a new app directory',
	},
	browseAppDirectory: {
		id: 'app.settings.resource-management.app-directory.browse',
		defaultMessage: 'Browse for an app directory',
	},
	appCacheTitle: {
		id: 'app.settings.resource-management.app-cache.title',
		defaultMessage: 'App cache',
	},
	purgeCache: {
		id: 'app.settings.resource-management.app-cache.purge',
		defaultMessage: 'Purge cache',
	},
	purgeCacheConfirmTitle: {
		id: 'app.settings.resource-management.app-cache.confirm.title',
		defaultMessage: 'Purge the app cache?',
	},
	purgeCacheConfirmDescription: {
		id: 'app.settings.resource-management.app-cache.confirm.description',
		defaultMessage: 'The app may load more slowly until the cache is rebuilt.',
	},
	appCacheDescription: {
		id: 'app.settings.resource-management.app-cache.description',
		defaultMessage:
			'Clear cached data and download it again from Modrinth. The app may load more slowly until the cache is rebuilt.',
	},
	maximumConcurrentDownloadsTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.title',
		defaultMessage: 'Maximum concurrent downloads',
	},
	maximumConcurrentDownloadsDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.description',
		defaultMessage:
			'Number of files the app can download at once. Lower this if downloads are unreliable on your connection. Requires an app restart.',
	},
	maximumConcurrentWritesTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.title',
		defaultMessage: 'Maximum concurrent writes',
	},
	maximumConcurrentWritesDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.description',
		defaultMessage:
			'Number of files the app can write to disk at once. Lower this if you frequently encounter I/O errors. Requires an app restart.',
	},
	alwaysShowCopyDetailsTitle: {
		id: 'app.settings.resource-management.always-show-copy-details.title',
		defaultMessage: 'Always show copy details',
	},
	alwaysShowCopyDetailsDescription: {
		id: 'app.settings.resource-management.always-show-copy-details.description',
		defaultMessage:
			'Show the Copy details action while an install is queued or running. It is always available for failed or interrupted installs.',
	},
	appDatabaseBackupsTitle: {
		id: 'app.settings.resource-management.app-database-backups.title',
		defaultMessage: 'App database backups',
	},
	openBackupsFolder: {
		id: 'app.settings.resource-management.app-database-backups.open-folder',
		defaultMessage: 'Open backups folder',
	},
	appDatabaseBackupsDescription: {
		id: 'app.settings.resource-management.app-database-backups.description',
		defaultMessage:
			'Backups of important app data are stored here in case you need to recover them later.',
	},
})

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)

async function purgeCache() {
	await purge_cache_types([
		'project',
		'project_v3',
		'version',
		'user',
		'team',
		'organization',
		'file',
		'loader_manifest',
		'minecraft_manifest',
		'categories',
		'report_types',
		'loaders',
		'game_versions',
		'donation_platforms',
		'file_hash',
		'file_update',
		'search_results',
		'search_results_v3',
	]).catch(handleError)
}

function handlePurgeCacheClick() {
	if (appSettings.getFeatureFlag('skip_non_essential_warnings')) {
		void purgeCache()
		return
	}

	purgeCacheConfirmModal.value?.show()
}

async function openDbBackupsFolder() {
	await showAppDbBackupsFolder().catch(handleError)
}

async function findLauncherDir() {
	const newDir = await open({
		multiple: false,
		directory: true,
		title: formatMessage(messages.selectAppDirectory),
	})

	if (newDir) {
		settings.value.custom_dir = newDir
	}
}
</script>

<template>
	<div>
		<ConfirmModalWrapper
			ref="purgeCacheConfirmModal"
			:title="formatMessage(messages.purgeCacheConfirmTitle)"
			:description="formatMessage(messages.purgeCacheConfirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.purgeCache)"
			:show-ad-on-close="false"
			@proceed="purgeCache"
		/>

		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup :label="formatMessage(messages.locationGroup)">
			<SettingsStack
				control-id="appDir"
				:title="formatMessage(messages.appDirectoryTitle)"
				:description="formatMessage(messages.appDirectoryDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Input
						:id="controlId"
						v-model="settings.custom_dir"
						:aria-labelledby="labelledBy"
						:icon="BoxIcon"
						type="text"
						wrapper-class="w-full"
					>
						<template #right>
							<IconButton
								v-tooltip="formatMessage(messages.browseAppDirectory)"
								:label="formatMessage(messages.browseAppDirectory)"
								class="ml-1.5"
								@click="findLauncherDir"
							>
								<FolderSearchIcon aria-hidden="true" />
							</IconButton>
						</template>
					</Input>
				</template>
			</SettingsStack>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.performanceGroup)">
			<SettingsStack
				control-id="max-downloads"
				:title="formatMessage(messages.maximumConcurrentDownloadsTitle)"
				:description="formatMessage(messages.maximumConcurrentDownloadsDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Slider
						:id="controlId"
						v-model="settings.max_concurrent_downloads"
						:aria-labelledby="labelledBy"
						:min="1"
						:max="10"
						:step="1"
					/>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="max-writes"
				:title="formatMessage(messages.maximumConcurrentWritesTitle)"
				:description="formatMessage(messages.maximumConcurrentWritesDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Slider
						:id="controlId"
						v-model="settings.max_concurrent_writes"
						:aria-labelledby="labelledBy"
						:min="1"
						:max="50"
						:step="1"
					/>
				</template>
			</SettingsStack>

			<SettingsRow
				control-id="always-show-copy-details"
				:title="formatMessage(messages.alwaysShowCopyDetailsTitle)"
				:description="formatMessage(messages.alwaysShowCopyDetailsDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						:model-value="appSettings.getFeatureFlag(alwaysShowCopyDetailsFlag)"
						:aria-labelledby="labelledBy"
						@update:model-value="
							() => {
								const newValue = !appSettings.getFeatureFlag(alwaysShowCopyDetailsFlag)
								appSettings.featureFlags[alwaysShowCopyDetailsFlag] = newValue
								settings.feature_flags[alwaysShowCopyDetailsFlag] = newValue
							}
						"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.maintenanceGroup)">
			<SettingsStack
				control-id="purge-cache"
				:title="formatMessage(messages.appCacheTitle)"
				:description="formatMessage(messages.appCacheDescription)"
			>
				<template #default="{ controlId }">
					<Button :id="controlId" class="w-fit" @click="handlePurgeCacheClick">
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.purgeCache) }}
					</Button>
				</template>
			</SettingsStack>

			<SettingsStack
				control-id="open-db-backups-folder"
				:title="formatMessage(messages.appDatabaseBackupsTitle)"
				:description="formatMessage(messages.appDatabaseBackupsDescription)"
			>
				<template #default="{ controlId }">
					<Button :id="controlId" class="w-fit" @click="openDbBackupsFolder">
						<FolderOpenIcon aria-hidden="true" />
						{{ formatMessage(messages.openBackupsFolder) }}
					</Button>
				</template>
			</SettingsStack>
		</SettingsGroup>
	</div>
</template>
