<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	useSavable,
} from '@modrinth/ui'
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import AccentColorSettings from '@/components/ui/settings/display/AccentColorSettings.vue'
import { useAccent } from '@/composables/use-accent.ts'
import { type ColorTheme, isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import type { AccentPresetId } from '@/helpers/accent-colors.ts'
import { type AppSettings, get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const theme = useTheme()
const accent = useAccent()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = inject(appSettingsModalContextKey, null)
const os = await getOS()
const settings = ref(await get())

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
	advancedRendering: boolean
	nativeDecorations: boolean
	accentPreset: AccentPresetId
	accentCustomHex: string
}

function getAppearanceSettingsState(settings: AppSettings): AppearanceSettingsState {
	const accentValue = accent.toSettingsValue(accent.saved)
	return {
		theme: settings.theme,
		syncAcrossDevices: settings.sync_theme_across_devices,
		advancedRendering: settings.advanced_rendering,
		nativeDecorations: settings.native_decorations,
		accentPreset: accentValue.preset,
		accentCustomHex: accentValue.customHex,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	() => getAppearanceSettingsState(settings.value),
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const nextSettings: AppSettings = {
			...settings.value,
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: value.advancedRendering,
			native_decorations: value.nativeDecorations,
		}

		await set(nextSettings)
		settings.value = nextSettings
		if (isDarkTheme(value.theme)) {
			theme.preferredDark = value.theme
		}
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = value.advancedRendering
		accent.save(
			accent.fromSettingsValue({
				preset: value.accentPreset,
				customHex: value.accentCustomHex,
			}),
		)
	},
)

const themeOptions = computed(() =>
	theme.options.filter(
		(option) =>
			option !== 'retro' || settings.value.developer_mode || current.value.theme === 'retro',
	),
)

const preferredDarkTheme = computed(() =>
	isDarkTheme(current.value.theme) ? current.value.theme : theme.preferredDark,
)

function setTheme(value: ColorTheme): void {
	current.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	current.value.advancedRendering = enabled
}

function setNativeDecorations(enabled: boolean): void {
	current.value.nativeDecorations = enabled
}

function setAccentPreset(preset: AccentPresetId): void {
	current.value.accentPreset = preset
	accent.setPreview(
		accent.fromSettingsValue({
			preset,
			customHex: current.value.accentCustomHex,
		}),
	)
}

function setAccentCustomHex(hex: string): void {
	current.value.accentCustomHex = hex
	if (current.value.accentPreset !== 'custom') {
		return
	}

	accent.setPreview(
		accent.fromSettingsValue({
			preset: 'custom',
			customHex: hex,
		}),
	)
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

watch(
	[
		() => current.value.accentPreset,
		() => current.value.accentCustomHex,
		() => saved.value.accentPreset,
		() => saved.value.accentCustomHex,
	],
	([preset, customHex, savedPreset, savedCustomHex]) => {
		if (preset === savedPreset && customHex === savedCustomHex) {
			accent.resetPreview()
			return
		}

		accent.setPreview(
			accent.fromSettingsValue({
				preset,
				customHex,
			}),
		)
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
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
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	accent.resetPreview()
	settingsModal?.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'MacOS'
			? {
					value: computed(() => current.value.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})
</script>

<template>
	<div class="flex flex-col gap-8">
		<AccentColorSettings
			:preset="current.accentPreset"
			:custom-hex="current.accentCustomHex"
			@update:preset="setAccentPreset"
			@update:custom-hex="setAccentCustomHex"
		/>
		<AppearanceSettingsLayout />
	</div>
</template>
