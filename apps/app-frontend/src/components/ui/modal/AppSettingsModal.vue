<script setup lang="ts">
import {
	CoffeeIcon,
	FolderOpenIcon,
	PaintbrushIcon,
	RefreshCwIcon,
	RocketIcon,
	Settings2Icon,
	ShieldIcon,
	ToggleRightIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	injectNotificationManager,
	ProgressBar,
	TabbedModal,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, provide, ref, watch } from 'vue'

import OctraMark from '@/components/brand/OctraMark.vue'
import OctraAccountSettings from '@/components/ui/settings/account/OctraAccountSettings.vue'
import PrivacySettings from '@/components/ui/settings/account/PrivacySettings.vue'
import AppearanceSettings from '@/components/ui/settings/display/AppearanceSettings.vue'
import BehaviorSettings from '@/components/ui/settings/display/BehaviorSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/display/FeatureFlagSettings.vue'
import InstancesSyncedSettings from '@/components/ui/settings/instances/InstancesSyncedSettings.vue'
import JavaSettings from '@/components/ui/settings/instances/JavaSettings.vue'
import LaunchDefaultsSettings from '@/components/ui/settings/instances/LaunchDefaultsSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/instances/ResourceManagementSettings.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get, set } from '@/helpers/settings.ts'
import {
	appSettingsModalContextKey,
	type UnsavedChangesController,
} from '@/providers/app-settings-modal'
import {
	appUpdateState,
	checkForAppUpdatesManually,
	type ManualUpdateCheckResult,
} from '@/providers/app-update.ts'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'

// TODO: Apply COMPONENT_STRUCTURE.md here and extract out common setting option components
const appSettings = useAppSettings()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabCategories = defineMessages({
	lookAndFeel: {
		id: 'app.settings.sidebar.label.look-and-feel',
		defaultMessage: 'Look & feel',
	},
	app: {
		id: 'app.settings.sidebar.label.app',
		defaultMessage: 'App',
	},
	gameDefaults: {
		id: 'app.settings.sidebar.label.game-defaults',
		defaultMessage: 'Game defaults',
	},
	system: {
		id: 'app.settings.sidebar.label.system',
		defaultMessage: 'System',
	},
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.lookAndFeel,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.lookAndFeel,
		icon: Settings2Icon,
		content: BehaviorSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.account',
			defaultMessage: 'Account',
		}),
		category: tabCategories.app,
		icon: UserIcon,
		content: OctraAccountSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		category: tabCategories.app,
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.launch',
			defaultMessage: 'Launch',
		}),
		category: tabCategories.gameDefaults,
		icon: RocketIcon,
		content: LaunchDefaultsSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.sync',
			defaultMessage: 'Sync',
		}),
		category: tabCategories.gameDefaults,
		icon: RefreshCwIcon,
		content: InstancesSyncedSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java',
		}),
		category: tabCategories.system,
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.storage',
			defaultMessage: 'Storage',
		}),
		category: tabCategories.system,
		icon: FolderOpenIcon,
		content: ResourceManagementSettings,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.system,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
]

const availableTabs = computed(() =>
	tabs.filter((tab) => !tab.developerOnly || appSettings.devMode),
)

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const unsavedChangesPopup = ref<{ nudge: () => void } | null>(null)
const unsavedChangesController = ref<UnsavedChangesController | null>(null)
const emptyUnsavedChangesState: Record<string, unknown> = {}
const originalUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getOriginal() ?? emptyUnsavedChangesState,
)
const modifiedUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getModified() ?? emptyUnsavedChangesState,
)
const savingUnsavedChanges = computed(() => unsavedChangesController.value?.isSaving() ?? false)
const hasUnsavedChanges = computed(
	() =>
		(unsavedChangesController.value?.hasChanges() ?? false) ||
		(unsavedChangesController.value?.isSaving() ?? false),
)

function canLeaveCurrentTab(): boolean {
	if (
		!unsavedChangesController.value?.hasChanges() &&
		!unsavedChangesController.value?.isSaving()
	) {
		return true
	}
	unsavedChangesPopup.value?.nudge()
	return false
}

function close(): boolean {
	return modal.value?.hide() ?? false
}

function registerUnsavedChangesController(controller: UnsavedChangesController | null): void {
	unsavedChangesController.value = controller
}

provide(appSettingsModalContextKey, {
	close,
	registerUnsavedChangesController,
})

function resetUnsavedChanges(): void {
	unsavedChangesController.value?.reset()
}

function saveUnsavedChanges(): void {
	void unsavedChangesController.value?.save()
}

function showTab(content: (typeof tabs)[number]['content']): void {
	const tabIndex = availableTabs.value.findIndex((tab) => tab.content === content)
	if (tabIndex >= 0) {
		modal.value?.setTab(tabIndex)
	}
	modal.value?.show()
}

function show() {
	modal.value?.show()
}

function showFeatureFlags(): void {
	showTab(FeatureFlagSettings)
}

function showAccount(): void {
	showTab(OctraAccountSettings)
}

function showSyncedOptions(): void {
	showTab(InstancesSyncedSettings)
}

function showLaunchDefaults(): void {
	showTab(LaunchDefaultsSettings)
}

defineExpose({
	show,
	showFeatureFlags,
	showAccount,
	showSyncedOptions,
	showLaunchDefaults,
})

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()
const { handleError, addNotification } = injectNotificationManager()
const { availableUpdate } = appUpdateState

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())
const checkingForUpdates = ref(false)

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		const selectedTab = modal.value ? availableTabs.value[modal.value.selectedTab] : undefined

		appSettings.devMode = !appSettings.devMode
		settings.value.developer_mode = !!appSettings.devMode
		devModeCounter.value = 0

		if (modal.value) {
			const selectedTabIndex = selectedTab ? availableTabs.value.indexOf(selectedTab) : -1
			modal.value.setTab(selectedTabIndex >= 0 ? selectedTabIndex : 0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	appVersion: {
		id: 'app.settings.app-version',
		defaultMessage: 'Octra App {version}',
	},
	macos: {
		id: 'app.settings.operating-system.macos',
		defaultMessage: 'macOS',
	},
	windows10: {
		id: 'app.settings.operating-system.windows-10',
		defaultMessage: 'Windows 10',
	},
	windows11: {
		id: 'app.settings.operating-system.windows-11',
		defaultMessage: 'Windows 11',
	},
	developerModeButtonLabel: {
		id: 'app.settings.developer-mode-button.label',
		defaultMessage: 'Toggle developer mode',
	},
	checkForUpdates: {
		id: 'app.settings.check-for-updates',
		defaultMessage: 'Check for updates',
	},
	checkingForUpdates: {
		id: 'app.settings.checking-for-updates',
		defaultMessage: 'Checking for updates…',
	},
	updatesDisabled: {
		id: 'app.settings.updates-disabled',
		defaultMessage: 'Automatic updates are only available in release builds.',
	},
	updateAlreadyLatest: {
		id: 'app.settings.update-already-latest',
		defaultMessage: 'You are on the latest version.',
	},
	updateAvailable: {
		id: 'app.settings.update-available',
		defaultMessage: 'Version {version} is available.',
	},
	updateCheckFailed: {
		id: 'app.settings.update-check-failed',
		defaultMessage: 'Could not check for updates. Try again later.',
	},
})

function windowsBuildNumber(raw: string): number {
	const build = Number.parseInt(raw.split('.')[2] ?? '', 10)
	return Number.isFinite(build) ? build : 0
}

const osLabel = computed(() => {
	if (osPlatform === 'macos') {
		return formatMessage(messages.macos)
	}
	if (osPlatform === 'windows') {
		return formatMessage(
			windowsBuildNumber(osVersion) >= 22000 ? messages.windows11 : messages.windows10,
		)
	}
	return osPlatform
})

async function handleCheckForUpdates() {
	if (checkingForUpdates.value) {
		return
	}

	checkingForUpdates.value = true
	let result: ManualUpdateCheckResult = 'error'

	try {
		result = await checkForAppUpdatesManually()
	} catch (error) {
		handleError(error)
	} finally {
		checkingForUpdates.value = false
	}

	if (result === 'disabled') {
		addNotification({
			type: 'info',
			title: formatMessage(messages.checkForUpdates),
			text: formatMessage(messages.updatesDisabled),
		})
		return
	}

	if (result === 'latest') {
		addNotification({
			type: 'success',
			title: formatMessage(messages.checkForUpdates),
			text: formatMessage(messages.updateAlreadyLatest),
		})
		return
	}

	if (result === 'error') {
		handleError(formatMessage(messages.updateCheckFailed))
		return
	}

	const updateVersion = availableUpdate.value?.version
	if (updateVersion) {
		addNotification({
			type: 'info',
			title: formatMessage(messages.checkForUpdates),
			text: formatMessage(messages.updateAvailable, { version: updateVersion }),
		})
	}
}
</script>
<template>
	<TabbedModal
		ref="modal"
		:tabs="availableTabs"
		:width="'min(928px, calc(95vw - 10rem))'"
		:before-hide="canLeaveCurrentTab"
		:before-tab-change="canLeaveCurrentTab"
		:floating-action-bar-shown="hasUnsavedChanges"
		variant="emberslot"
		nav-layout="top"
	>
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(commonMessages.settingsLabel) }}
			</span>
		</template>
		<template #floating-action-bar>
			<UnsavedChangesPopup
				ref="unsavedChangesPopup"
				:original="originalUnsavedChangesState"
				:modified="modifiedUnsavedChangesState"
				:saving="savingUnsavedChanges"
				inline
				@reset="resetUnsavedChanges"
				@save="saveUnsavedChanges"
			/>
		</template>
		<template #footer>
			<div class="app-settings-modal__footer mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<p v-if="appSettings.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex flex-wrap items-center gap-3">
					<button
						:aria-label="formatMessage(messages.developerModeButtonLabel)"
						class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
						:class="{
							'text-brand': appSettings.devMode,
							'text-secondary': !appSettings.devMode,
						}"
						@click="devModeCount"
					>
						<OctraMark aria-hidden="true" class="h-6 w-6" />
					</button>
					<div class="min-w-0">
						<p class="m-0">
							{{ formatMessage(messages.appVersion, { version }) }}
						</p>
						<p class="m-0">
							{{ osLabel }}
						</p>
					</div>
					<Button
						size="sm"
						type="outlined"
						:disabled="checkingForUpdates"
						@click="handleCheckForUpdates"
					>
						<RefreshCwIcon :class="{ 'animate-spin': checkingForUpdates }" />
						{{
							checkingForUpdates
								? formatMessage(messages.checkingForUpdates)
								: formatMessage(messages.checkForUpdates)
						}}
					</Button>
				</div>
			</div>
		</template>
	</TabbedModal>
</template>

<style lang="scss" scoped>
.app-settings-modal__footer {
	border-top: 1px solid var(--surface-5);
	margin-top: 0.75rem;
	padding-top: 0.75rem;
	background: transparent;
}
</style>
