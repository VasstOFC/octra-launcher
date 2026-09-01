export const ACCENT_PRESET_IDS = ['octra', 'cobalt', 'ember', 'mint', 'rose', 'aurora'] as const

export type AccentPresetId = (typeof ACCENT_PRESET_IDS)[number] | 'custom'

export interface AccentPreference {
	preset: AccentPresetId
	customHex: string
}

export interface AccentPresetDefinition {
	id: Exclude<AccentPresetId, 'custom'>
	light: string
	dark: string
}

export const ACCENT_PRESETS: readonly AccentPresetDefinition[] = [
	{ id: 'octra', light: '#a051a2', dark: '#c47bc6' },
	{ id: 'cobalt', light: '#1f68c0', dark: '#5196df' },
	{ id: 'ember', light: '#e08325', dark: '#e7a038' },
	{ id: 'mint', light: '#00af5c', dark: '#33f598' },
	{ id: 'rose', light: '#ed4661', dark: '#f67687' },
	{ id: 'aurora', light: '#761ad6', dark: '#ba7eff' },
]

export const DEFAULT_ACCENT: AccentPreference = {
	preset: 'octra',
	customHex: '#a051a2',
}

const BRAND_CSS_VARS = [
	'--color-brand',
	'--color-brand-highlight',
	'--color-brand-shadow',
	'--brand-gradient-bg',
	'--brand-gradient-strong-bg',
	'--brand-gradient-border',
	'--brand-gradient-fade-out-color',
	'--loading-bar-gradient',
	'--splash-tint-top',
	'--splash-tint-bottom',
	'--splash-overlay',
	'--color-purple-highlight',
	'--color-focus-ring',
] as const

export function parseHexColor(hex: string): { r: number; g: number; b: number } | null {
	const normalized = hex.trim().replace(/^#/, '')
	if (!/^[0-9a-fA-F]{6}$/.test(normalized)) {
		return null
	}

	return {
		r: Number.parseInt(normalized.slice(0, 2), 16),
		g: Number.parseInt(normalized.slice(2, 4), 16),
		b: Number.parseInt(normalized.slice(4, 6), 16),
	}
}

export function normalizeHexColor(hex: string, fallback = DEFAULT_ACCENT.customHex): string {
	const parsed = parseHexColor(hex)
	if (!parsed) {
		return fallback
	}

	const toHex = (value: number) => value.toString(16).padStart(2, '0')
	return `#${toHex(parsed.r)}${toHex(parsed.g)}${toHex(parsed.b)}`
}

function rgba(hex: string, alpha: number): string {
	const parsed = parseHexColor(hex)
	if (!parsed) {
		return `rgba(160, 81, 162, ${alpha})`
	}

	return `rgba(${parsed.r}, ${parsed.g}, ${parsed.b}, ${alpha})`
}

function mixHex(base: string, target: string, amount: number): string {
	const a = parseHexColor(base)
	const b = parseHexColor(target)
	if (!a || !b) {
		return base
	}

	const mix = (from: number, to: number) => Math.round(from + (to - from) * amount)
	const toHex = (value: number) => value.toString(16).padStart(2, '0')
	return `#${toHex(mix(a.r, b.r))}${toHex(mix(a.g, b.g))}${toHex(mix(a.b, b.b))}`
}

function lightenHex(hex: string, amount: number): string {
	return mixHex(hex, '#ffffff', amount)
}

function darkenHex(hex: string, amount: number): string {
	return mixHex(hex, '#000000', amount)
}

export function resolveAccentHex(preference: AccentPreference, isDark: boolean): string {
	if (preference.preset === 'custom') {
		const custom = normalizeHexColor(preference.customHex)
		return isDark ? lightenHex(custom, 0.12) : custom
	}

	const preset = ACCENT_PRESETS.find((entry) => entry.id === preference.preset)
	if (!preset) {
		return isDark ? DEFAULT_ACCENT.customHex : DEFAULT_ACCENT.customHex
	}

	return isDark ? preset.dark : preset.light
}

export function buildBrandCssVariables(brandHex: string, isDark: boolean): Record<string, string> {
	const secondary = isDark ? lightenHex(brandHex, 0.18) : lightenHex(brandHex, 0.22)
	const deep = isDark ? darkenHex(brandHex, 0.45) : darkenHex(brandHex, 0.35)
	const fade = isDark ? darkenHex(brandHex, 0.55) : lightenHex(brandHex, 0.72)

	return {
		'--color-brand': brandHex,
		'--color-brand-highlight': rgba(brandHex, 0.25),
		'--color-brand-shadow': rgba(brandHex, 0.7),
		'--brand-gradient-bg': isDark
			? `linear-gradient(0deg, ${rgba(deep, 0.55)} 0%, ${rgba(brandHex, 0.18)} 100%)`
			: `linear-gradient(0deg, ${rgba(brandHex, 0.175)} 0%, ${rgba(secondary, 0.125)} 100%)`,
		'--brand-gradient-strong-bg': isDark
			? `linear-gradient(270deg, ${rgba(deep, 0.95)} 10%, ${rgba(darkenHex(brandHex, 0.25), 0.75)} 100%)`
			: `linear-gradient(270deg, ${rgba(brandHex, 0.175)} 0%, ${rgba(secondary, 0.12)} 100%)`,
		'--brand-gradient-border': isDark ? rgba(lightenHex(brandHex, 0.25), 0.08) : rgba(deep, 0.15),
		'--brand-gradient-fade-out-color': isDark
			? `linear-gradient(to bottom, rgba(24, 30, 31, 0), ${rgba(deep, 0.85)} 80%)`
			: `linear-gradient(to bottom, ${rgba(fade, 0)}, ${rgba(fade, 0.95)} 70%)`,
		'--loading-bar-gradient': `linear-gradient(to right, ${brandHex} 0%, ${secondary} 100%)`,
		'--splash-tint-top': rgba(isDark ? lightenHex(brandHex, 0.08) : lightenHex(brandHex, 0.35), isDark ? 0.28 : 0.46),
		'--splash-tint-bottom': rgba(isDark ? deep : lightenHex(brandHex, 0.2), isDark ? 0.5 : 0.56),
		'--splash-overlay': rgba(isDark ? deep : lightenHex(brandHex, 0.35), isDark ? 0.64 : 0.32),
		'--color-purple-highlight': rgba(brandHex, 0.25),
		'--color-focus-ring': lightenHex(brandHex, isDark ? 0.35 : 0.4),
	}
}

export function applyBrandCssVariables(preference: AccentPreference, isDark: boolean): void {
	const brandHex = resolveAccentHex(preference, isDark)
	const variables = buildBrandCssVariables(brandHex, isDark)
	const html = document.documentElement

	for (const [name, value] of Object.entries(variables)) {
		html.style.setProperty(name, value)
	}
}

export function clearBrandCssVariables(): void {
	const html = document.documentElement
	for (const name of BRAND_CSS_VARS) {
		html.style.removeProperty(name)
	}
}

export function accentPreferencesEqual(a: AccentPreference, b: AccentPreference): boolean {
	return a.preset === b.preset && normalizeHexColor(a.customHex) === normalizeHexColor(b.customHex)
}
