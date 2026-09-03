import { prepareThemeColorTransition } from '@modrinth/ui'
import { reactive, ref, watch } from 'vue'

import { isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import {
	type AccentPreference,
	type AccentPresetId,
	applyBrandCssVariables,
	DEFAULT_ACCENT,
	normalizeHexColor,
} from '@/helpers/accent-colors.ts'

const ACCENT_STORAGE_KEY = 'octra.accent'

export type AccentSettingsValue = {
	preset: AccentPresetId
	customHex: string
}

function loadAccentPreference(): AccentPreference {
	try {
		const raw = window.localStorage.getItem(ACCENT_STORAGE_KEY)
		if (!raw) {
			return { ...DEFAULT_ACCENT }
		}

		const parsed = JSON.parse(raw) as Partial<AccentPreference>
		if (!parsed.preset) {
			return { ...DEFAULT_ACCENT }
		}

		return {
			preset: parsed.preset,
			customHex: normalizeHexColor(parsed.customHex ?? DEFAULT_ACCENT.customHex),
		}
	} catch {
		return { ...DEFAULT_ACCENT }
	}
}

function persistAccentPreference(preference: AccentPreference): void {
	try {
		window.localStorage.setItem(
			ACCENT_STORAGE_KEY,
			JSON.stringify({
				preset: preference.preset,
				customHex: normalizeHexColor(preference.customHex),
			}),
		)
	} catch {
		// storage blocked or full
	}
}

const saved = ref<AccentPreference>(loadAccentPreference())
const preview = ref<AccentPreference | null>(null)

function getActivePreference(): AccentPreference {
	return preview.value ?? saved.value
}

function applyAccent(preference: AccentPreference, withTransition = false): void {
	const theme = useTheme()
	const isDark = isDarkTheme(theme.active)

	if (withTransition) {
		prepareThemeColorTransition()
	}

	applyBrandCssVariables(preference, isDark)
}

function initAccent(): void {
	applyAccent(saved.value)
}

const accent = reactive({
	saved,
	preview,
	get preference() {
		return getActivePreference()
	},
	applySaved(withTransition = false) {
		preview.value = null
		applyAccent(saved.value, withTransition)
	},
	setPreview(preference: AccentPreference | null) {
		preview.value = preference
		applyAccent(getActivePreference(), true)
	},
	save(preference: AccentPreference) {
		const normalized: AccentPreference = {
			preset: preference.preset,
			customHex: normalizeHexColor(preference.customHex),
		}
		saved.value = normalized
		preview.value = null
		persistAccentPreference(normalized)
		applyAccent(normalized, true)
	},
	resetPreview() {
		preview.value = null
		applyAccent(saved.value, true)
	},
	toSettingsValue(preference: AccentPreference = getActivePreference()): AccentSettingsValue {
		return {
			preset: preference.preset,
			customHex: normalizeHexColor(preference.customHex),
		}
	},
	fromSettingsValue(value: AccentSettingsValue): AccentPreference {
		return {
			preset: value.preset,
			customHex: normalizeHexColor(value.customHex),
		}
	},
	loadFromStorage: loadAccentPreference,
})

watch(
	() => useTheme().active,
	() => {
		applyAccent(getActivePreference())
	},
)

export function useAccent() {
	return accent
}

export function bootstrapAccent(): void {
	initAccent()
}
