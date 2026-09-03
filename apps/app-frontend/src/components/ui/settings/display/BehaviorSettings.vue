<script setup lang="ts">
import {
	defineMessages,
	injectAuth,
	injectUserPreferences,
	Toggle,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { inject, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	SettingsGroup,
	SettingsPanelHeader,
	SettingsRow,
} from '@/components/ui/settings/_shared'
import {
	DEFAULT_FEATURE_FLAGS,
	type FeatureFlag,
	useAppSettings,
} from '@/composables/use-app-settings.ts'
import { type AppSettings, get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const appSettings = useAppSettings()
const { formatMessage } = useVIntl()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = inject(appSettingsModalContextKey, null)

const worldsInHomeFlag: FeatureFlag = 'worlds_in_home'
const compactInstanceCardsFlag: FeatureFlag = 'compact_instance_cards'
const skipNonEssentialWarningsFlag: FeatureFlag = 'skip_non_essential_warnings'
const skipUnknownPackWarningFlag: FeatureFlag = 'skip_unknown_pack_warning'
const showPlayTimeFlag: FeatureFlag = 'show_instance_play_time'

const messages = defineMessages({
	panelTitle: {
		id: 'app.settings.behavior.panel.title',
		defaultMessage: 'Behavior',
	},
	panelDescription: {
		id: 'app.settings.behavior.panel.description',
		defaultMessage: 'Startup, home content, confirmations, and Discord status.',
	},
	syncAcrossDevicesTitle: {
		id: 'app.behavior-settings.sync-across-devices.title',
		defaultMessage: 'Sync behavior across devices',
	},
	syncAcrossDevicesDescription: {
		id: 'app.behavior-settings.sync-across-devices.description',
		defaultMessage:
			"Use these behavior settings everywhere you're signed in. Turn this off to keep separate settings on this device.",
	},
	syncAcrossDevicesSignedOutTooltip: {
		id: 'app.behavior-settings.sync-across-devices.signed-out-tooltip',
		defaultMessage: "Octra accounts are coming later. You'll be able to sync settings then.",
	},
	syncGroup: {
		id: 'app.behavior-settings.group.sync',
		defaultMessage: 'Sync',
	},
	startupAndNavigationTitle: {
		id: 'app.behavior-settings.startup-and-navigation.title',
		defaultMessage: 'Startup and navigation',
	},
	contentTitle: {
		id: 'app.behavior-settings.content.title',
		defaultMessage: 'Home and content',
	},
	confirmationsTitle: {
		id: 'app.behavior-settings.confirmations.title',
		defaultMessage: 'Confirmations',
	},
	integrationsTitle: {
		id: 'app.behavior-settings.integrations.title',
		defaultMessage: 'Integrations',
	},
	minimizeLauncherTitle: {
		id: 'app.appearance-settings.minimize-launcher.title',
		defaultMessage: 'Minimize app',
	},
	minimizeLauncherDescription: {
		id: 'app.appearance-settings.minimize-launcher.description',
		defaultMessage: 'Minimize Octra App when Minecraft starts.',
	},
	jumpBackIntoWorldsTitle: {
		id: 'app.appearance-settings.jump-back-into-worlds.title',
		defaultMessage: 'Jump into worlds or instances',
	},
	jumpBackIntoWorldsDescription: {
		id: 'app.appearance-settings.jump-back-into-worlds.description',
		defaultMessage:
			'Show recently played worlds or instances in the "Jump in" section on the Home page.',
	},
	compactModeTitle: {
		id: 'app.appearance-settings.compact-mode.title',
		defaultMessage: 'List layout',
	},
	compactModeDescription: {
		id: 'app.appearance-settings.compact-mode.description',
		defaultMessage:
			'Show library instances as compact list rows. Turn off for a quieter card grid.',
	},
	showPlayTimeTitle: {
		id: 'app.appearance-settings.show-play-time.title',
		defaultMessage: 'Show play time',
	},
	showPlayTimeDescription: {
		id: 'app.appearance-settings.show-play-time.description',
		defaultMessage: `Show how long you've played each instance.`,
	},
	hideNametagTitle: {
		id: 'app.appearance-settings.hide-nametag.title',
		defaultMessage: 'Hide nametag',
	},
	hideNametagDescription: {
		id: 'app.appearance-settings.hide-nametag.description',
		defaultMessage: 'Hide your username above the player preview on the Skin selector page.',
	},
	unknownPackWarningTitle: {
		id: 'app.appearance-settings.unknown-pack-warning.title',
		defaultMessage: 'Warn me before installing unknown modpacks',
	},
	unknownPackWarningDescription: {
		id: 'app.appearance-settings.unknown-pack-warning.description',
		defaultMessage:
			"Show a safety warning before installing a Modrinth Pack (.mrpack) that isn't hosted on Modrinth.",
	},
	skipNonEssentialWarningsTitle: {
		id: 'app.appearance-settings.skip-non-essential-warnings.title',
		defaultMessage: 'Skip non-essential warnings',
	},
	skipNonEssentialWarningsDescription: {
		id: 'app.appearance-settings.skip-non-essential-warnings.description',
		defaultMessage:
			'Skip confirmations for low-risk actions such as duplicate installs, normal content deletion, bulk updates, unlinking, and repairs. Warnings for dangerous actions are always shown.',
	},
	discordRichPresenceTitle: {
		id: 'app.settings.privacy.discord-rich-presence.title',
		defaultMessage: 'Discord Rich Presence',
	},
	discordRichPresenceDescription: {
		id: 'app.settings.privacy.discord-rich-presence.description',
		defaultMessage:
			'Show Octra App as your current activity on Discord. This does not affect Rich Presence added to instances by mods. Requires an app restart.',
	},
})

type BehaviorSettingsState = {
	syncBehaviorAcrossDevices: boolean
	minimizeApp: boolean
	showJumpIn: boolean
	compactInstanceCards: boolean
	showPlayTime: boolean
	hideNametag: boolean
	warnOnUnknownModpacks: boolean
	skipNonEssentialWarnings: boolean
	discordRpc: boolean
}

const persistedSettings = ref(await get())

function getBehaviorSettingsState(settings: AppSettings): BehaviorSettingsState {
	return {
		syncBehaviorAcrossDevices: settings.sync_behavior_across_devices,
		minimizeApp: settings.hide_on_process_start,
		showJumpIn: settings.feature_flags[worldsInHomeFlag] ?? DEFAULT_FEATURE_FLAGS[worldsInHomeFlag],
		compactInstanceCards:
			settings.feature_flags[compactInstanceCardsFlag] ??
			DEFAULT_FEATURE_FLAGS[compactInstanceCardsFlag],
		showPlayTime:
			settings.feature_flags[showPlayTimeFlag] ?? DEFAULT_FEATURE_FLAGS[showPlayTimeFlag],
		hideNametag: settings.hide_nametag_skins_page,
		warnOnUnknownModpacks: !(
			settings.feature_flags[skipUnknownPackWarningFlag] ??
			DEFAULT_FEATURE_FLAGS[skipUnknownPackWarningFlag]
		),
		skipNonEssentialWarnings:
			settings.feature_flags[skipNonEssentialWarningsFlag] ??
			DEFAULT_FEATURE_FLAGS[skipNonEssentialWarningsFlag],
		discordRpc: settings.discord_rpc,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	() => getBehaviorSettingsState(persistedSettings.value),
	async () => {
		const value = current.value

		if (value.syncBehaviorAcrossDevices && auth.user.value) {
			await updatePreferences({
				behavior: {
					minimize_app: value.minimizeApp,
					show_jump_in: value.showJumpIn,
					compact_instance_cards: value.compactInstanceCards,
					show_play_time: value.showPlayTime,
					hide_nametag: value.hideNametag,
					warn_on_unknown_modpacks: value.warnOnUnknownModpacks,
					skip_non_essential_warnings: value.skipNonEssentialWarnings,
				},
			})
		}

		const nextSettings: AppSettings = {
			...persistedSettings.value,
			sync_behavior_across_devices: value.syncBehaviorAcrossDevices,
			hide_on_process_start: value.minimizeApp,
			toggle_sidebar: true,
			hide_nametag_skins_page: value.hideNametag,
			discord_rpc: value.discordRpc,
			feature_flags: {
				...persistedSettings.value.feature_flags,
				[worldsInHomeFlag]: value.showJumpIn,
				[compactInstanceCardsFlag]: value.compactInstanceCards,
				[showPlayTimeFlag]: value.showPlayTime,
				[skipUnknownPackWarningFlag]: !value.warnOnUnknownModpacks,
				[skipNonEssentialWarningsFlag]: value.skipNonEssentialWarnings,
			},
		}

		await set(nextSettings)
		persistedSettings.value = nextSettings
		appSettings.setBehaviorSyncAcrossDevices(value.syncBehaviorAcrossDevices)
		appSettings.hideNametagSkinsPage = value.hideNametag
		appSettings.featureFlags[worldsInHomeFlag] = value.showJumpIn
		appSettings.featureFlags[compactInstanceCardsFlag] = value.compactInstanceCards
		appSettings.featureFlags[showPlayTimeFlag] = value.showPlayTime
		appSettings.featureFlags[skipUnknownPackWarningFlag] = !value.warnOnUnknownModpacks
		appSettings.featureFlags[skipNonEssentialWarningsFlag] = value.skipNonEssentialWarnings
	},
)

async function saveBehaviorSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveBehaviorSettings,
	})
})

onBeforeUnmount(() => {
	settingsModal?.registerUnsavedChangesController(null)
})
</script>

<template>
	<div class="flex flex-col">
		<SettingsPanelHeader
			:title="formatMessage(messages.panelTitle)"
			:description="formatMessage(messages.panelDescription)"
		/>

		<SettingsGroup :label="formatMessage(messages.syncGroup)">
			<SettingsRow
				control-id="sync-behavior-across-devices"
				:title="formatMessage(messages.syncAcrossDevicesTitle)"
				:description="formatMessage(messages.syncAcrossDevicesDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<span
						v-tooltip="
							!auth.user.value
								? formatMessage(messages.syncAcrossDevicesSignedOutTooltip)
								: undefined
						"
						class="inline-flex shrink-0"
					>
						<Toggle
							:id="controlId"
							:model-value="Boolean(auth.user.value) && current.syncBehaviorAcrossDevices"
							:disabled="!auth.user.value"
							:aria-labelledby="labelledBy"
							@update:model-value="current.syncBehaviorAcrossDevices = $event"
						/>
					</span>
				</template>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.startupAndNavigationTitle)">
			<SettingsRow
				control-id="minimize-launcher"
				:title="formatMessage(messages.minimizeLauncherTitle)"
				:description="formatMessage(messages.minimizeLauncherDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.minimizeApp"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.contentTitle)">
			<SettingsRow
				control-id="jump-back-into-worlds"
				:title="formatMessage(messages.jumpBackIntoWorldsTitle)"
				:description="formatMessage(messages.jumpBackIntoWorldsDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.showJumpIn"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>

			<SettingsRow
				control-id="compact-mode"
				:title="formatMessage(messages.compactModeTitle)"
				:description="formatMessage(messages.compactModeDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.compactInstanceCards"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>

			<SettingsRow
				control-id="show-play-time"
				:title="formatMessage(messages.showPlayTimeTitle)"
				:description="formatMessage(messages.showPlayTimeDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.showPlayTime"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>

			<SettingsRow
				control-id="hide-nametag-skins-page"
				:title="formatMessage(messages.hideNametagTitle)"
				:description="formatMessage(messages.hideNametagDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.hideNametag"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.confirmationsTitle)">
			<SettingsRow
				control-id="warn-before-installing-unknown-modpacks"
				:title="formatMessage(messages.unknownPackWarningTitle)"
				:description="formatMessage(messages.unknownPackWarningDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.warnOnUnknownModpacks"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>

			<SettingsRow
				control-id="skip-non-essential-warnings"
				:title="formatMessage(messages.skipNonEssentialWarningsTitle)"
				:description="formatMessage(messages.skipNonEssentialWarningsDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.skipNonEssentialWarnings"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>

		<SettingsGroup :label="formatMessage(messages.integrationsTitle)">
			<SettingsRow
				control-id="disable-discord-rpc"
				:title="formatMessage(messages.discordRichPresenceTitle)"
				:description="formatMessage(messages.discordRichPresenceDescription)"
			>
				<template #default="{ labelledBy, controlId }">
					<Toggle
						:id="controlId"
						v-model="current.discordRpc"
						:aria-labelledby="labelledBy"
					/>
				</template>
			</SettingsRow>
		</SettingsGroup>
	</div>
</template>
